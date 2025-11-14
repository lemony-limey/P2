use tokio::sync::watch;
use anyhow::Context;
use log::debug;
use std::{
    thread,
    time::Duration
};
use tokio::sync::{broadcast, mpsc, oneshot};

#[tokio::main]
async fn main() -> anyhow::Result<()>
{
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();

    // Create a channel that will be used to transfer received messages
    let (tx_messages, rx_messages) = tokio::sync::mpsc::channel::<String>(100);
    debug!("created channel for sending messages to user's thread");

    // Create a channel which will be used to terminate the beacon application
    // when the user exists the UI.
    let (tx_beacon_shutdown, mut rx_beacon_shutdown) = broadcast::channel::<()>(16);
    debug!("created channel for sending messages");

    // Spin up a thread for the UI.
    filetree_browser::run(rx_messages);
    // let ui_handle = thread::spawn(|| {
    //
    // });


    // Spin up a separate thread for the beacon application.
    let beacon_handle = thread::spawn(move || {

    });


    // When the UI exits, shutdown the beacon application also.
    tx_beacon_shutdown.send(()).expect("sending beacon tx shutdown message failed");
    beacon_handle.join().expect("beacon graceful shutdown failed");

    Ok(())
}