//! Multicast beacon application.
//! Rust version based on saleem's original Java

pub mod configuration;
pub mod multicast;

use crate::message_socket::MessageSocket;
use tokio::sync::RwLock;
use anyhow::{anyhow, Context};
use configuration::MulticastConfiguration;
use crate::{
    protocol::{
        discovery::AdvertisementMessage,
        identifier::{UID, FQDN, Identifier},
        message::{Header, Timestamp, Payload, Message},
        process_status::ProcessStatus,
        serial_number::SerialNumber,
        services::{DownloadAvailability, SearchAvailability, Services},
    },
};
use log::info;
use multicast::{MulticastEndpoint, PacketType};
use std::{
    sync::mpsc::Receiver,
    time::Duration,
    sync::{Arc},
    num::{NonZeroU64, NonZeroU16}
};
use tokio::{
    sync::{Mutex, mpsc, watch},
};
use tokio::time::Interval;

pub async fn run(
    rx_shutdown: tokio::sync::watch::Receiver<ProcessStatus>,
) -> anyhow::Result<()>
{
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();
    info!("Starting Beacon application");

    let mut config = MulticastConfiguration::load("beacon.toml")?;
    let username = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "user".to_string());
    config.username = username;

    let mut endpoint = MulticastEndpoint::new(config)?;
    endpoint.join()?;
    let endpoint = Arc::new(Mutex::new(endpoint));

    // Spawn the receiving thread. This is a listener that will receive messages as they come in,
    // parse them, and return them to the main thread for storage.

    // let mut threads = tokio::task::JoinSet::new();
    // threads.j


    info!("Beacon application terminated");

    Ok(())
}


/// Send a beacon ping out every so often
pub async fn beacon(beacon: Arc<RwLock<MessageSocket>>) -> anyhow::Result<()>
{
    // Predefine all info that does not have to be redefined later in the critical section.
    let beacon_interval: Duration;
    let identifier: Identifier;
    let payload: Payload;
    {
        let beacon_handle = beacon.read().await;
        let endpoint = beacon_handle.endpoint();
        beacon_interval = Duration::from_millis(endpoint.config.beacon_interval);
        identifier = Identifier::new(
            UID(endpoint.config.username.clone()),
            FQDN(endpoint.config.host_name.clone()),
        );
        payload = Payload::Advertisement(
            AdvertisementMessage::new(
                NonZeroU16::new(endpoint.config.m_port)
                    .with_context(|| "beacon(): invalid port number")?,
                Services::new(
                    // TODO: Use the actual values instead of these placeholders.
                    SearchAvailability(true),
                    DownloadAvailability(true),
                )
            )
        );
    }

    // The beacon will send an advertisement message once every [interval] milliseconds.
    let mut interval = tokio::time::interval(beacon_interval);
    loop
    {
        // Get the next serial number from the configuration.
        let next_serial_number: NonZeroU64;
        {
            next_serial_number = beacon.write().await.endpoint_mut().config.get_next_serial_number();
        }

        // Header must be different each iteration because timestamp is constantly changing.
        let message = Message::new(
            Header::new(
                identifier.clone(),
                SerialNumber(next_serial_number.clone()),
                Timestamp::now()
                    .with_context(|| "beacon(): could not create timestamp with supplied values")?,
            ),
            payload.clone()
        );

        // Send the beacon message.
        {
            beacon.read().await.tx_message(message).await?;
        }

        // Pause to avoid excess traffic.
        interval.tick().await;
    }
}
