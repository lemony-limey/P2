use log::error;
use log::warn;
use chrono::DateTime;
use crate::protocol::identifier::Identifier;
use chrono::{TimeDelta, Utc};
use std::net::{IpAddr, SocketAddr, SocketAddrV6};
use std::ops::Add;
use log::debug;
use std::path::PathBuf;
use crate::soft_state::{FileMetadata, Node};
use crate::protocol::message::{Message, Payload, ResponseID, Timestamp};
use crate::protocol::message::Header;
use crate::message_socket::MessageSocket;
use anyhow::{anyhow};
use std::sync::Arc;
use std::time::Duration;
use log::info;
use tokio::sync::RwLock;
use crate::constants::{TTL_NANOSECONDS, TTL_SECONDS};
use crate::filetree_browser::FileTreeBrowser;
use crate::logical_file_path::LogicalFilePath;
use crate::multicast::multicast_endpoint::PacketType;
use crate::protocol::download::DownloadMessage;
use crate::protocol::search::SearchMessage;
use crate::protocol::serial_number::SerialNumber;
use crate::simple_tcp::client::run_client;
use crate::soft_state::FTBSoftState;

pub mod multicast;
pub mod constants;
pub mod filetree_browser;
pub mod logical_file_path;
pub mod message_socket;
pub mod n_digit_number;
pub mod parser;
pub mod protocol;
pub mod simple_tcp;
pub mod soft_state;

pub async fn process_incoming_messages(
    socket: MessageSocket,
    soft_state: Arc<RwLock<FTBSoftState>>,
    ftb: Arc<RwLock<FileTreeBrowser>>,
) -> anyhow::Result<()>
{
    info!("starting listener");

    // Process incoming messages
    loop
    {
        let result = socket.rx_message().await?;

        let (header, payload, rx_addr): (Header, Payload, SocketAddr);

        match result
        {
            // Await next packet
            Some((PacketType::Ipv4, received_from_addr, received_message)) => {
                info!("->rx4 from:{}, {}", &received_from_addr, &received_message);
                (header, payload, rx_addr) = (received_message.header, received_message.payload, received_from_addr);
            },
            Some((PacketType::Ipv6, received_from_addr, received_message)) => {
                info!("->rx6 from:{}, {}", &received_from_addr, &received_message);
                (header, payload, rx_addr) = (received_message.header, received_message.payload, received_from_addr);
            },
            Some(_) | None => {
                info!("packet dropped");
                continue;
            },
        }

        // Match the type of the message to determine how to process it
        match payload
        {
            // If advertisement, put into storage
            Payload::Advertisement(advertisement_message) => {
                // Attempt to make the node, drop the packet if it fails.
                let Ok(node) = Node::new(
                    header.identifier,
                    rx_addr,
                    advertisement_message.services,
                ) else { continue; };

                // Add the node to the soft state.
                info!("requesting soft_state write lock");
                soft_state.write().await.discovery_update(node).await?;
            },
            // If search, query storage then send message if no results have been returned.
            Payload::Search(SearchMessage::Request { search_string }) => {
                // Check search is enabled
                if !ftb.read().await.config.search
                {
                    debug!("search request dropped: search is disabled");
                    continue;
                }

                // Check if any file within the logical file directory has a substring match
                // with the search_string.
                let matches: Vec<LogicalFilePath>;
                {
                    matches = ftb.read().await.search_for_substring(&search_string).await?;
                }

                // Spawn a new task for responding so that tokio::time::sleep does not stop
                // the whole listener thread.
                let ftb = ftb.clone();
                let socket = socket.clone();
                tokio::spawn(async move {
                    let header_identifier = header.identifier.clone();
                    let header_serial_number = header.serial_number.clone();

                    // If there are no matches, ignore the request
                    // Else, send a search response out for every match found.
                    for file_match in matches
                    {
                        let serial_number: SerialNumber;
                        let identifier: Identifier;
                        let file_size: u64;
                        {
                            let mut ftb_handle = ftb.write().await;
                            serial_number = SerialNumber(ftb_handle.config.next_search_serial_number());
                            identifier = ftb_handle.get_current_identifier();
                            if let Ok(match_file_size) = ftb_handle.get_file_size(&file_match).await
                            {
                                file_size = match_file_size;
                            }
                            else
                            {
                                error!("could not get file size of \"{}\"", &file_match);
                                continue;
                            }
                        }

                        let search_response = Message::new(
                            Header::new(
                                identifier.clone(),
                                serial_number,
                                Timestamp::now().expect("could not get current timestamp"),
                            ),
                            Payload::Search(
                                SearchMessage::Result {
                                    response_id: ResponseID::new(
                                        header_identifier.clone(),
                                        header_serial_number,
                                    ),
                                    file_path: file_match,
                                    file_size,
                                },
                            ),
                        );

                        // As per Multicast DNS (RFC 6762), Section 6.3
                        tokio::time::sleep(Duration::from_millis(rand::random_range(20..=120))).await;
                        let _ = socket.tx_message(search_response).await;
                    }
                });  // End of tokio::spawn block
            },
            // If this is a response to a search message initiated by the current user, add the
            // file name and size to the database for the node sending the response.
            // The other nodes should only ever respond if they do contain a file that matches
            // the pattern provided.
            Payload::Search(SearchMessage::Result { response_id, file_path: file_string, file_size }) => {
                // Add file to the node, if it exists
                let soft_state_handle = soft_state.write().await;
                let mut nodes_handle = soft_state_handle.nodes.write().await;

                let Some(node) = nodes_handle.get_mut(response_id.identifier()) else {
                    debug!("search result dropped: unknown node");
                    continue;
                };

                // Get a write lock on the node reporting that is has the specified file
                let mut node_handle = node.write().await;
                let mut ttl = Utc::now();
                let Some(delta) = TimeDelta::new(TTL_SECONDS, TTL_NANOSECONDS) else {
                    return Err(anyhow!("an error occurred when attempting to create the TTL timestamp"));
                };
                ttl = ttl.add(delta);

                // Convert file_string to logical path and store with ttl
                node_handle.files.insert(
                    LogicalFilePath::new(file_string),
                    FileMetadata::new(
                        file_size,
                        ttl,
                    ),
                );
            },
            // If download, check if download is enabled then query storage,
            // then eventually spin up the TCP server.
            // TCP will be done asynchronously.
            Payload::Download(DownloadMessage::Request { identifier, logical_file_path: file_path }) => {
                // Ensure the download request is actually for this endpoint
                if ftb.read().await.get_current_identifier() != identifier
                {
                    info!("download request dropped: not intended for this node");
                    continue;
                }

                // Ensure download is enabled.
                if !ftb.read().await.config.download
                {
                    info!("download request dropped: download is disabled");
                    continue;
                }

                // Check if the requested file is in storage by making a call to FileTreeBrowser,
                // which handles local files. It will return a PathBuf if found.
                {
                    if !ftb.read().await.does_file_exist(&file_path).await?
                    {
                        info!(
                            "download request dropped: requested file \"{}\" does not exist",
                            &file_path,
                        );
                        continue;
                    }
                }

                // Get the address of the host requesting the file transfer.
                let path_to_file: PathBuf;
                let target_addr: SocketAddr;
                // Lock the soft state cache.
                {
                    let soft_state_handle = soft_state.read().await;
                    let nodes_handle = soft_state_handle.nodes.read().await;
                    if let Some(target_host) = nodes_handle.get(&header.identifier) {
                        // Update its TTL value
                        let mut target_host_handle = target_host.write().await;
                        target_host_handle.ttl = get_ttl_timestamp(TTL_SECONDS, TTL_NANOSECONDS)?;
                        target_addr = target_host_handle.address;
                    }
                    else
                    {
                        info!("download request dropped: target host not found");
                        continue;
                    };
                }
                // Lock the file tree browser.
                {
                    let ftb_handle = ftb.read().await;
                    let Ok(path) = ftb_handle.logical_path_to_absolute_path(&file_path).await else {
                        warn!("download request dropped: conversion from logical path to absolute path failed");
                        continue;
                    };
                    path_to_file = path;
                }

                // Start TCP client connection and attempt transfer.
                info!("starting TCP connection");

                let SocketAddr::V6(target_addr) = target_addr else {
                    error!("download request dropped: download must be over ipv6");
                    continue;
                };

                if let Err(e) = run_client(
                    target_addr,
                    path_to_file,
                ).await
                {
                    error!("download error: {}", e);
                    continue;
                }
            },
            // A host is indicating to us that another host has successfully received a
            // download.
            // Incoming file string will be treated as a logical path.
            Payload::Download(DownloadMessage::Result { logical_file_path, file_size }) => {
                // Add file to the node, if it exists
                let soft_state_handle = soft_state.write().await;
                let mut nodes_handle = soft_state_handle.nodes.write().await;

                // Get a mutable handle to the node
                let Some(node) = nodes_handle.get_mut(&header.identifier) else {
                    info!("search result dropped: unknown node");
                    continue;
                };

                // Add the file along with a time to live value.
                let ttl = get_ttl_timestamp(TTL_SECONDS, TTL_NANOSECONDS)?;
                let mut node_handle = node.write().await;

                // Convert file_string to logical path
                node_handle.files.insert(
                    LogicalFilePath::new(logical_file_path),
                    FileMetadata::new(
                        file_size,
                        ttl,
                    ),
                );
            },
            // Remove specified entry from the cache
            Payload::Leaving => {
                let soft_state_handle_clone = soft_state.clone();

                // Spawn deletion in a separate thread to avoid blocking the listener.
                tokio::spawn(async move {
                    let soft_state_handle = soft_state_handle_clone.write().await;
                    let mut nodes_handle = soft_state_handle.nodes.write().await;

                    // As per RFC 6762 (Multicast DNS), Section 10.1 - Goodbye Packets
                    tokio::time::sleep(Duration::from_secs(1)).await;

                    nodes_handle.remove(&header.identifier);
                });
            }
        }
    }
}

pub fn get_ttl_timestamp(seconds: i64, nanoseconds: u32) -> anyhow::Result<DateTime<Utc>>
{
    let mut ttl = Utc::now();
    let Some(delta) = TimeDelta::new(seconds, nanoseconds) else {
        return Err(anyhow!("an error occurred when attempting to create the TTL timestamp"));
    };
    ttl = ttl.add(delta);

    Ok(ttl)
}
