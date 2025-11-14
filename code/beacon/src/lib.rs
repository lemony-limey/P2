//! Multicast beacon application.
//! Rust version based on saleem's original Java

mod configuration;
mod multicast;

use parser::protocol::message::Header;
use parser::protocol::message::Payload;
use tokio::sync::mpsc::Sender;
use anyhow::Context;
use configuration::Configuration;
use log::info;
use multicast::{MulticastEndpoint, PacketType};
use parser::protocol::{
    message::Message,
    process_status::ProcessStatus,
};
use std::{
    time::Duration,
    sync::{Arc},
};
use std::io::Read;
use std::num::NonZeroU16;
use tokio::{
    sync::{
        Mutex,
        mpsc,
        watch,
    },
};
use bytes::{Bytes, BytesMut};
use parser::parser::message::parse_message;
use parser::protocol::discovery::AdvertisementMessage;
use parser::protocol::services::{DownloadAvailability, SearchAvailability, Services};

const TX_RATIO: u32 = 5;

pub async fn run(
    tx_to_filetree_browser: mpsc::Sender<Message<'_>>,
    rx_shutdown: watch::Receiver<ProcessStatus>,
) -> anyhow::Result<()>
{
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();

    info!("Starting Beacon application");

    let mut config = Configuration::load("beacon.toml")?;
    let username = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "user".to_string());
    config.username = username;

    let mut endpoint = MulticastEndpoint::new(config)?;
    endpoint.join()?;

    let mut endpoint = Arc::new(Mutex::new(endpoint));

    // Spawn the beacon. It will periodically send
    tokio::spawn(async move {
        let result = periodically_send_beacon(
            endpoint.clone(),
        ).await;
    });

    // Spawn the receiving thread. This is a listener that will receive messages as they come in,
    // parse them, and return them to the main thread for storage.

    let mut endpoint = endpoint.lock().await;
    endpoint.leave()?;
    info!("Beacon application terminated");

    Ok(())
}

// TODO: Only pass necessary config info instead of locking the whole struct.
async fn periodically_send_beacon(
    endpoint: Arc<Mutex<MulticastEndpoint>>,
) -> anyhow::Result<()>
{
    let sleep_time: Duration;
    let payload: Payload;
    {
        let config_handle = endpoint.lock().await.config();
        sleep_time = Duration::from_millis(config_handle.beacon_interval);
        payload = Payload::new(
            AdvertisementMessage::new(
                NonZeroU16::new(config_handle.m_port)
                    .with_context(|| "periodically_send_beacon(): invalid port number")?,
                Services::new(
                    SearchAvailability(config_handle),
                    DownloadAvailability(),
                )
            )
        );
    }

    let mut interval = tokio::time::interval(sleep_time);


    loop
    {
        // Header must be different because timestamp is constantly changing.
        let message = Message::new(
            Header::new(

            ),
            payload
        );

        // transmit a beacon periodically
        tx_message(message, &mut endpoint)?;

        // pause to avoid excess traffic
        interval.tick().await;
    }
}

/// Used by the main thread to send stuff with tx_beacon, which is a private method.
pub async fn multicast_tx(
    rx_
) -> anyhow::Result<()>
{

}

/// Receives and processes an incoming beacon.
///
/// Will attempt to parse the message and perform the appropriate action.
async fn rx_beacon(
    endpoint: &mut MulticastEndpoint,
    tx_to_user: &mut Sender<Message<'_>>, // Send the received messages to filetree-browser
) -> anyhow::Result<()>
{
    let mut buffer = BytesMut::with_capacity(endpoint.config.msg_size);

    match endpoint.rx(&mut buffer).await?
    {
        PacketType::None => Ok(()),
        PacketType::Ipv4 => {
            // let message_string = String::from_utf8_lossy(&buffer)
            //     .trim_end_matches('\0')
            //     .trim()
            //     .to_string();
            let message = parse_message(&buffer)?;
            info!("->rx4 {}", &message);
            tx_to_user.send(message).await?;
            Ok(())
        },
        PacketType::Ipv6 => {
            let message = parse_message(&Bytes::from(
                String::from_utf8_lossy(&buffer)
                    .trim_end_matches('\0')
                    .trim()
            ))?;
            info!("->rx6 {}", &message);
            tx_to_user.send(message).await?;
            Ok(())
        },
    }
}

/// transmits a beacon message.
async fn tx_message(
    message: Message<'_>,
    endpoint: &mut MulticastEndpoint,
) -> anyhow::Result<()>
{
    // send IPv4 beacon if configured
    if !endpoint.config().m_addr4.is_empty()
    {
        if endpoint.tx(PacketType::Ipv4, &message.to_string().as_bytes()).await?
        {
            info!("tx4-> {}", &message);
        }
    }

    // send IPv6 beacon if configured
    if !endpoint.config().m_addr6.is_empty()
    {
        if endpoint.tx(PacketType::Ipv6, &message.to_string().as_bytes()).await?
        {
            info!("tx6-> {}", &message);
        }
    }

    Ok(())
}
