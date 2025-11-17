use tokio::sync::RwLock;
use std::sync::Arc;
use crate::message_socket::MessageSocket;

pub mod beacon;
pub mod constants;
pub mod filetree_browser;
pub mod message_socket;
pub mod n_digit_number;
pub mod parser;
pub mod protocol;
pub mod simple_tcp;
pub mod ui;
mod soft_state;

pub async fn process_messages(socket: Arc<RwLock<MessageSocket>>) -> anyhow::Result<()>
{
    // Process incoming messages
    while let Ok(message) = socket.read().await.rx_message().await
    {
        // If advertisement, put into storage

        // If search, query storage

        // If download, check if download is enabled then query storage,
        // then eventually spin up the TCP server.

    }

    Ok(())
}
