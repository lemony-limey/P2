use code::constants::{FILETREE_BROWSER_CONFIGURATION_FILE, MULTICAST_CONFIGURATION_FILE};
use code::filetree_browser::{configuration::FiletreeBrowserConfiguration, run_ftb_ui, FileTreeBrowser};
use code::message_socket::MessageSocket;
use code::multicast::beacon;
use code::multicast::multicast_configuration::MulticastConfiguration;
use code::multicast::multicast_endpoint::MulticastEndpoint;
use code::soft_state::FTBSoftState;
use code::process_incoming_messages;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::fs;
use code::protocol::services::{DownloadAvailability, SearchAvailability};

#[tokio::main]
async fn main() -> anyhow::Result<()>
{
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();

    // Get multicast configuration
    let config = get_multicast_configuration(MULTICAST_CONFIGURATION_FILE)?;
    let mut endpoint = MulticastEndpoint::new(config)?;
    endpoint.join()?;
    let socket = Arc::new(RwLock::new(MessageSocket::new(endpoint)));

    // Get filetree browser configuration and create the filetree browser.
    let state = Arc::new(RwLock::new(FTBSoftState::new()));
    let ftb_config = FiletreeBrowserConfiguration::load(FILETREE_BROWSER_CONFIGURATION_FILE)?;
    let root_path = fs::canonicalize(&ftb_config.root_dir).await?;
    let ftb = Arc::new(RwLock::new(FileTreeBrowser::new(
        &root_path,
        root_path.clone(),
        ftb_config.clone(),
        socket.clone(),
        state.clone(),
    )));

    // Spin up a thread to run the beacon program.
    // TODO: Potentially blocking thread or std thread here
    let socket_handle = socket.clone();
    tokio::spawn(async move {
        beacon(
            socket_handle,
            SearchAvailability(ftb_config.search),
            DownloadAvailability(ftb_config.download),
        ).await
    });

    // Process all incoming messages
    let socket_handle = socket.clone();
    let ftb_handle = ftb.clone();
    tokio::spawn(async move {
        process_incoming_messages(socket_handle, state.clone(), ftb_handle).await
    });

    // Spin up a thread for the UI.
    run_ftb_ui(ftb).await?;

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
