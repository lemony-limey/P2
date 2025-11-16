//! Multicast beacon application.
//! Rust version based on saleem's original Java

mod configuration;
mod multicast;

use std::sync::mpsc::Receiver;
use anyhow::{anyhow, Context};
use configuration::Configuration;
use log::info;
use multicast::{MulticastEndpoint, PacketType};
use parser::{
    protocol::{
        discovery::AdvertisementMessage,
        identifier::{UID, FQDN, Identifier},
        message::{Header, Timestamp, Payload, Message},
        process_status::ProcessStatus,
        serial_number::SerialNumber,
        services::{DownloadAvailability, SearchAvailability, Services},
    },
};
use std::{
    time::Duration,
    sync::{Arc},
    num::{NonZeroU64, NonZeroU16},
};
use tokio::{
    sync::{Mutex, mpsc, watch},
};

pub async fn run(
    tx_to_user_main: tokio::sync::mpsc::Sender<Message>,
    rx_from_user_main: tokio::sync::mpsc::Receiver<Message>,
    rx_shutdown: tokio::sync::watch::Receiver<ProcessStatus>,
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
    let endpoint = Arc::new(Mutex::new(endpoint));

    // Spawn the beacon. It will periodically send discovery messages.
    let endpoint_handle = endpoint.clone();
    let beacon_thread = tokio::spawn(async move {
        beacon(endpoint_handle).await
    });

    // Spawn the multicast receiver thread
    let endpoint_handle = endpoint.clone();
    let listener_handle = tokio::spawn(async move {
        multicast_listener(endpoint_handle, tx_to_user_main).await
    });

    // TODO: Spawn the multicast transmission thread
    let endpoint_handle = endpoint.clone();
    let transmitter_handle = tokio::spawn(async move {
        multicast_transmitter(endpoint_handle, rx_from_user_main).await
    });


    // Spawn the receiving thread. This is a listener that will receive messages as they come in,
    // parse them, and return them to the main thread for storage.

    // let mut threads = tokio::task::JoinSet::new();
    // threads.j

    // Cleanup
    let mut endpoint = endpoint.lock().await;
    endpoint.leave()?;
    info!("Beacon application terminated");

    Ok(())
}

/// Send a beacon ping out every so often
async fn beacon(
    endpoint: Arc<Mutex<MulticastEndpoint>>,
) -> anyhow::Result<()>
{
    // Predefine all info that does not have to be redefined later in the critical section.
    let beacon_interval: Duration;
    let identifier: Identifier;
    let payload: Payload;
    {
        let mut endpoint = endpoint.lock().await;
        let config = endpoint.config_mut();
        beacon_interval = Duration::from_millis(config.beacon_interval);
        identifier = Identifier::new(
            UID(config.username.clone()),
            FQDN(config.host_name.clone()),
        );
        payload = Payload::Advertisement(
            AdvertisementMessage::new(
                NonZeroU16::new(config.m_port)
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
            let mut endpoint = endpoint.lock().await;
            let config = endpoint.config_mut();
            next_serial_number = config.get_next_serial_number();
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

        // transmit a beacon periodically
        tx_message(message, endpoint.clone()).await?;

        // pause to avoid excess traffic
        interval.tick().await;
    }
}

/// Used by the main thread to send stuff with multicast.
/// tx_beacon is a private method, so this is used to call tx_beacon whenever a
pub async fn multicast_transmitter(
    endpoint: Arc<Mutex<MulticastEndpoint>>,
    mut rx_messages_for_transmission: tokio::sync::mpsc::Receiver<Message>,
) -> anyhow::Result<()>
{
    // Loop and receive messages from the main thread while the channel is open.
    while let Some(message) = rx_messages_for_transmission.recv().await
    {
        // Send the message.
        tx_message(message, endpoint.clone()).await?
    }

    Ok(())
}

/// Listens for incoming messages
pub async fn multicast_listener(
    endpoint: Arc<Mutex<MulticastEndpoint>>,
    tx_to_user: mpsc::Sender<Message>, // Send the received messages to filetree-browser
) -> anyhow::Result<()>
{
    // Keep receiving messages for as long as the program is running.
    // TODO: Check for any instances of ? that would cause this to exit prematurely.
    while let Ok(message) = rx_message(endpoint.clone()).await
    {
        tx_to_user.send(message).await?;
    }

    Ok(())
}

/// Receives and processes an incoming multicast datagram.
///
/// Will attempt to parse the message and perform the appropriate action.
async fn rx_message(
    endpoint: Arc<Mutex<MulticastEndpoint>>,
) -> anyhow::Result<Message>
{
    let (packet_type, message): (PacketType, Option<Message>);
    {
        let mut endpoint = endpoint.lock().await;
        if let Ok((received_packet_type, received_message)) = endpoint.rx().await {
            packet_type = received_packet_type;
            message = Some(received_message);
        }
        else
        {
            packet_type = PacketType::None;
            message = None;
        }
    }

    match (packet_type, message)
    {
        (PacketType::None, _) => Err(anyhow!("an error occurred when reading the packet")),
        (PacketType::Ipv4, Some(message)) => {
            info!("->rx4 {}", &message);
            Ok(message)
        },
        (PacketType::Ipv6, Some(message)) => {
            info!("->rx6 {}", &message);
            Ok(message)
        },
        (_, _) => {
            Err(anyhow!("rx_message(): invalid combination of message and packet type"))
        }
    }
}

/// Transmits a single UDP datagram over multicast, via IPv4 AND IPv6 (meaning at most two packets
/// are sent overall).
async fn tx_message(
    message: Message,
    endpoint: Arc<Mutex<MulticastEndpoint>>,
) -> anyhow::Result<()>
{
    let mut endpoint = endpoint.lock().await;

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
