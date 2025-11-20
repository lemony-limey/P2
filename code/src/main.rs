use log::info;
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
use tokio::task::{spawn_blocking, JoinSet};
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
    info!("{:#?}", endpoint);
    let message_socket = MessageSocket::new(endpoint);

    // Get filetree browser configuration and create the filetree browser.
    let state = Arc::new(RwLock::new(FTBSoftState::new()));
    let ftb_config = FiletreeBrowserConfiguration::load(FILETREE_BROWSER_CONFIGURATION_FILE)?;
    let root_path = fs::canonicalize(&ftb_config.root_dir).await?;
    let ftb = Arc::new(RwLock::new(FileTreeBrowser::new(
        &root_path,
        root_path.clone(),
        ftb_config.clone(),
        message_socket.clone(),
        state.clone(),
    )));

    // Spin up a thread to run the beacon program.
    // let socket_handle_1 = socket.clone();
    // tokio::spawn(async move {
    //     beacon(
    //         socket_handle_1,
    //         SearchAvailability(ftb_config.search),
    //         DownloadAvailability(ftb_config.download),
    //     ).await
    // });

    // Process all incoming messages
    // let socket_handle_2 = socket.clone();
    // let ftb_handle = ftb.clone();
    // let state_handle = state.clone();
    // tokio::spawn(async move {
    //     process_incoming_messages(socket_handle_2, state_handle, ftb_handle).await
    // });
    //
    // // Spin up a thread for the UI.
    // let ftb_handle = ftb.clone();
    // tokio::spawn(async move {
    //     run_ftb_ui(ftb_handle).await
    // });
    //
    let cloned_socket_1 = message_socket.clone();
    let cloned_socket_2 = message_socket.clone();
    let ftb_handle = ftb.clone();
    let state_handle = state.clone();
    tokio::select! {
        result = tokio::spawn(async move {
            beacon(
                cloned_socket_1,
                SearchAvailability(ftb_config.search),
                DownloadAvailability(ftb_config.download),
            ).await
        }) => {
            info!("beacon returned");
            println!("{:?}", result);
        },
        result = tokio::spawn(async move {
            process_incoming_messages(
                cloned_socket_2,
                state_handle,
                ftb_handle
            ).await
        }) => {
            info!("listener returned");
            println!("{:?}", result);
        },
        // thing = run_ftb_ui(ftb) => {
        //     info!("ui returned");
        //     println!("{:?}", thing);
        // }
    };

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
