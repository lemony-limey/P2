use crate::protocol::identifier::Identifier;
use chrono::{TimeDelta, Utc};
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use std::num::NonZeroU16;
use std::ops::Add;
use log::debug;
use std::path::PathBuf;
use crate::soft_state::Node;
use crate::protocol::message::{Message, Payload, ResponseID, Timestamp};
use crate::protocol::message::Header;
use crate::message_socket::MessageSocket;
use anyhow::{anyhow, Context};
use std::sync::Arc;
use log::info;
use tokio::sync::RwLock;
use crate::constants::{TTL_NANOSECONDS, TTL_SECONDS};
use crate::filetree_browser::configuration::FiletreeBrowserConfiguration;
use crate::filetree_browser::FileTreeBrowser;
use crate::logical_file_path::LogicalFilePath;
use crate::multicast::multicast_endpoint::PacketType;
use crate::protocol::download::DownloadMessage;
use crate::protocol::identifier::{FQDN, UID};
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
pub mod ui;

pub async fn process_incoming_messages(
    socket: Arc<RwLock<MessageSocket>>,
    soft_state: Arc<RwLock<FTBSoftState>>,
    ftb: Arc<RwLock<FileTreeBrowser>>,
) -> anyhow::Result<()>
{
    // Process incoming messages
    loop
    {
        let result: (PacketType, Option<Message>);
        {
            result = socket.read().await.rx_message().await?;
        }

        let (header, payload): (Header, Payload);
        match result
        {
            // Await next packet
            (PacketType::None, None) => { continue },
            (PacketType::Ipv4, Some(received_message)) => {
                info!("->rx4 {}", &received_message);
                (header, payload) = (received_message.header, received_message.payload);
            },
            (PacketType::Ipv6, Some(received_message)) => {
                info!("->rx6 {}", &received_message);
                (header, payload) = (received_message.header, received_message.payload);
            },
            (_, _) => {
                return Err(anyhow!("rx_message(): an error occurred when receiving a message"))
            }
        }

        // Match the type of the message to determine how to process it
        // TODO: When we hear from a node we know, update its TTL value.
        match payload
        {
            // If advertisement, put into storage
            Payload::Advertisement(advertisement_message) => {
                let address = SocketAddr::new(
                    IpAddr::V6(advertisement_message.ipv6_link_local_addr),
                    advertisement_message.server_port.get(),
                );

                // Attempt to make the node, drop the packet if it fails.
                let Ok(node) = Node::new(
                    header.identifier,
                    address,
                    advertisement_message.services,
                ) else { continue; };

                // Add the node to the soft state.
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

                // If there are no matches, ignore the request
                // Else, send a search response out for every match found.
                // TODO: Perhaps introduce rate limiting?
                for file_match in matches
                {
                    let serial_number: SerialNumber;
                    let uid: UID;
                    let fqdn: FQDN;
                    // Split into 2 blocks to minimise the time we hold the exclusive write() handle for.
                    {
                        let mut ftb_handle = ftb.write().await;
                        serial_number = SerialNumber(ftb_handle.config.next_search_serial_number());
                    }
                    {
                        let ftb_handle = ftb.read().await;
                        uid = UID(ftb_handle.config.uid.to_string());
                        fqdn = FQDN(ftb_handle.config.fqdn.to_string());
                    }

                    let search_response = Message::new(
                        Header::new(
                            Identifier::new(
                                uid,
                                fqdn,
                            ),
                            serial_number,
                            Timestamp::now()?,
                        ),
                        Payload::Search(
                            SearchMessage::Result {
                                response_id: ResponseID::new(
                                    header.identifier.clone(),
                                    header.serial_number.clone(),
                                ),
                                file_string: file_match,
                            },
                        ),
                    );

                    socket.write().await.tx_message(search_response).await?;
                }
            },
            // If this is a response to a search message initiated by the current user, add the
            // file name and size to the database for the node sending the response.
            // The other nodes should only ever respond if they do contain a file that matches
            // the pattern provided.
            Payload::Search(SearchMessage::Result { response_id, file_string }) => {
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
                node_handle.files.insert(LogicalFilePath::new(file_string), ttl);
            },
            // If download, check if download is enabled then query storage,
            // then eventually spin up the TCP server.
            // TCP will be done asynchronously.
            Payload::Download(DownloadMessage::Request { identifier, logical_file_path: file_path }) => {
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
                {
                    let soft_state_handle = soft_state.read().await;
                    let nodes_handle = soft_state_handle.nodes.read().await;
                    let Some(target_host) = nodes_handle.get(&header.identifier) else {
                        info!("download request dropped: target host not found");
                        continue;
                    };
                    let target_host_handle = target_host.read().await;
                    target_addr = target_host_handle.address;
                }
                {
                    let ftb_handle = ftb.read().await;
                    let Ok(path) = ftb_handle.logical_path_to_absolute_path(file_path).await else {
                        info!("download request dropped: conversion from logical path to absolute path failed");
                        continue;
                    };
                    path_to_file = path;
                }

                info!("starting TCP connection");
                // Start TCP client connection and attempt transfer.
                run_client(
                    target_addr,
                    path_to_file,
                ).await?
            },
            // A host is indicating to us that another host has successfully received a
            // download.
            // Incoming file string will be treated as a logical path.
            Payload::Download(DownloadMessage::Result { response_id, logical_file_path: file_string }) => {
                // Add file to the node, if it exists
                let soft_state_handle = soft_state.write().await;
                let mut nodes_handle = soft_state_handle.nodes.write().await;

                // Get a mutable handle to the node
                let Some(node) = nodes_handle.get_mut(response_id.identifier()) else {
                    debug!("search result dropped: unknown node");
                    continue;
                };

                // Add the file along with a time to live value.
                let mut node_handle = node.write().await;
                let mut ttl = Utc::now();
                let Some(delta) = TimeDelta::new(TTL_SECONDS, TTL_NANOSECONDS) else {
                    return Err(anyhow!("an error occurred when attempting to create the TTL timestamp"));
                };
                ttl = ttl.add(delta);

                // Convert file_string to logical path
                node_handle.files.insert(LogicalFilePath::new(file_string), ttl);
            },
            // Remove specified entry from the cache
            Payload::Leaving => {
                let soft_state_handle = soft_state.write().await;
                let mut nodes_handle = soft_state_handle.nodes.write().await;

                nodes_handle.remove(&header.identifier);
            }
        }
    }
}
