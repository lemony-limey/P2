use tokio::task::JoinSet;
use std::sync::Arc;
use anyhow::Context;
use code::{process_messages, protocol::{
    message::Message,
    process_status::ProcessStatus,
}};
use log::{debug, info};
use std::{
    thread,
    time::Duration,
};
use tokio::{task::spawn_blocking, sync::{broadcast, mpsc, watch, oneshot}, fs};
use tokio::sync::RwLock;
use code::multicast::beacon;
use code::multicast::multicast_configuration::MulticastConfiguration;
use code::multicast::multicast_endpoint::MulticastEndpoint;
use code::constants::{FILETREE_BROWSER_CONFIGURATION_FILE, MULTICAST_CONFIGURATION_FILE};
use code::filetree_browser::{FileTreeBrowser, configuration::FiletreeBrowserConfiguration, run_ftb};
use code::message_socket::MessageSocket;
use code::soft_state::SoftState;

#[tokio::main]
async fn main() -> anyhow::Result<()>
{
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();

    let config = get_multicast_configuration(MULTICAST_CONFIGURATION_FILE)?;
    let mut endpoint = MulticastEndpoint::new(config)?;
    endpoint.join()?;
    let socket = Arc::new(RwLock::new(MessageSocket::new(endpoint)));

    let storage = Arc::new(RwLock::new(SoftState::new()));
    let ftb_config = FiletreeBrowserConfiguration::load(FILETREE_BROWSER_CONFIGURATION_FILE)?;

    let root_path = fs::canonicalize(&ftb_config.root_dir).await?;
    let mut ftb = FileTreeBrowser::new(
        &root_path,
        root_path.clone(),
        ftb_config,
        socket.clone(),
        storage,
    );

    // Spin up a thread to run the beacon program.
    let socket_handle = socket.clone();
    tokio::spawn(async move {
        beacon(socket_handle).await
    });

    // Process all incoming messages
    let socket_handle = socket.clone();
    tokio::spawn(async move {
        process_messages(socket.clone()).await
    });

    // Spin up a thread for the UI.
    run_ftb(&mut ftb).await?;

    // On exit, tokio will stop the beacon and message processing tasks.
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
