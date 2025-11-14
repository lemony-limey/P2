use anyhow::Context;
use beacon;
use log::debug;
use parser::protocol::{
    message::Message,
    process_status::ProcessStatus,
};
use std::{
    thread,
    time::Duration,
};
use tokio::sync::{broadcast, mpsc, watch, oneshot};


#[tokio::main]
async fn main() -> anyhow::Result<()>
{
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();

    // Create a channel that will be used to transfer received messages
    let (tx_passive_listener, rx_passive_listener) = tokio::sync::mpsc::channel::<Message>(100);
    debug!("created channel for sending messages to user's thread");

    // TODO: Should potentially be a standard library mpsc since it is sending to sync code.
    let (tx_multicast_sender, rx_multicast_sender) = tokio::sync::mpsc::channel::<Message>(100);

    // Create a channel which will be used to terminate the beacon application
    // when the user exists the UI.
    let (tx_shutdown, rx_shutdown) = watch::channel::<ProcessStatus>(ProcessStatus::Running);
    debug!("created channel for sending messages");

    // Spin up a thread for the UI.

    // Spin up a separate thread for the beacon application.
    let beacon_handle = tokio::spawn(async move {
        beacon::run(tx_passive_listener.clone(), rx_shutdown.clone())
    }).join();

    // Spin up a TCP listener thread.



    // When the UI exits, shutdown the beacon application also.
    filetree_browser::run(rx_passive_listener)?;
    tx_shutdown.send(ProcessStatus::Shutdown).expect("sending beacon tx shutdown message failed");
    beacon_handle.await.expect("beacon graceful shutdown failed");

    Ok(())
}