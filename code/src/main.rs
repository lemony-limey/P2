use tokio::task::JoinSet;
use std::sync::Arc;
use anyhow::Context;
use code::{process_messages, protocol::{
    message::Message,
    process_status::ProcessStatus,
}};
use log::debug;
use std::{
    thread,
    time::Duration,
};
use tokio::{
    task::spawn_blocking,
    sync::{broadcast, mpsc, watch, oneshot}
};
use tokio::sync::RwLock;
use code::beacon::beacon;
use code::beacon::configuration::MulticastConfiguration;
use code::beacon::multicast::MulticastEndpoint;
use code::constants::BEACON_CONFIGURATION_FILE;
use code::message_socket::MessageSocket;

#[tokio::main]
async fn main() -> anyhow::Result<()>
{
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();

    let config = get_multicast_configuration(BEACON_CONFIGURATION_FILE)?;
    let mut endpoint = MulticastEndpoint::new(config)?;
    endpoint.join()?;

    let socket = Arc::new(RwLock::new(MessageSocket::new(endpoint)));

    // Spin up a thread to run the beacon program.
    let socket_handle = socket.clone();
    let beacon_handle = tokio::spawn(async move {
        beacon(socket_handle).await
    });

    // Process all incoming messages
    let socket_handle = socket.clone();
    let processor_handle = tokio::spawn(async move {
        process_messages(socket_handle).await
    });

    // Spin up a thread for the UI.

    // When the UI exits, shutdown the beacon application also.

    // Join all handles
    let (beacon, processor) = tokio::join!(
        beacon_handle,
        processor_handle,
    );

    Ok(())
}

fn get_multicast_configuration(beacon_config_file: &str) -> anyhow::Result<MulticastConfiguration>
{
    let mut config = MulticastConfiguration::load(beacon_config_file)?;
    let username = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "user".to_string());
    config.username = username;

    Ok(config)
}
