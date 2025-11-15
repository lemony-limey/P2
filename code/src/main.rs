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

    // Create a channel that will be used to transfer received messages from the multicast
    // listener thread to the main user thread.
    let (tx_received_messages, rx_received_messages) = tokio::sync::mpsc::channel::<Message>(100);
    debug!("created channel for sending messages to user's thread");

    // Create a channel that will be used to transfer messages to the multicast transmission thread.
    let (tx_to_multicast_sender, rx_multicast_sender) = tokio::sync::mpsc::channel::<Message>(100);

    // Create a channel which will be used to terminate the beacon application
    // when the user exists the UI.
    let (tx_shutdown, rx_shutdown) = watch::channel::<ProcessStatus>(ProcessStatus::Running);
    debug!("created channel for sending messages");


    // Spin up a thread to run the beacon program.
    let multicast_listener_handle = tokio::spawn(async move {
        let result = beacon::run(
            tx_received_messages.clone(),
            rx_shutdown.clone()
        ).await;
        result
    });

    // Spin up a thread for the UI.

    // Spin up a TCP listener thread.

    // Initialise storage

    // When the UI exits, shutdown the beacon application also.
    filetree_browser::run(rx_received_messages).await?;
    tx_shutdown.send(ProcessStatus::Shutdown).expect("sending beacon tx shutdown message failed");
    multicast_listener_handle.await.expect("beacon graceful shutdown failed")?;

    Ok(())
}