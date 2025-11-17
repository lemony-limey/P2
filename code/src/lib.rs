use crate::protocol::message::Payload;
use crate::protocol::message::Header;
use crate::message_socket::MessageSocket;
use anyhow::{anyhow, Context};
use std::sync::Arc;
use log::info;
use tokio::sync::RwLock;
use crate::multicast::multicast_endpoint::PacketType;

pub mod multicast;
pub mod constants;
pub mod filetree_browser;
pub mod message_socket;
pub mod n_digit_number;
pub mod parser;
pub mod protocol;
pub mod simple_tcp;
pub mod soft_state;
pub mod ui;

pub async fn process_messages(socket: Arc<RwLock<MessageSocket>>) -> anyhow::Result<()>
{
    // Process incoming messages
    loop
    {
        let result;
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
        match payload
        {
            // If advertisement, put into storage
            Payload::Advertisement(advertisement_message) => {

            },
            // If search, query storage then send message if no results have been returned.
            Payload::Search(search_message) => {

            },
            // If download, check if download is enabled then query storage,
            // then eventually spin up the TCP server.
            Payload::Download(download_message) => {

            },
        }
    }
}
