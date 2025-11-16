//! Multicast beacon application.
//! Rust version based on saleem's original Java

pub mod configuration;
pub mod multicast;

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

struct Beacon
{
    endpoint: MulticastEndpoint,
    /// Beacon uses tx_incoming_messages to send the received messages to the user, and
    /// rx_incoming_messages is used by the main thread to receive these messages.
    tx_incoming_messages: Option<mpsc::Sender<Message>>,
    rx_incoming_messages: Option<mpsc::Receiver<Message>>,
}

impl Beacon
{
    pub fn new(config: MulticastConfiguration) -> anyhow::Result<Self>
    {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
            .format_timestamp_millis()
            .init();
        info!("Starting Beacon application");

        let mut endpoint = MulticastEndpoint::new(config)?;
        endpoint.join()?;

        Ok(
            Self
            {
                endpoint,
                tx_incoming_messages: None,
                rx_incoming_messages: None,
            }
        )
    }

    /// Send a beacon ping out every so often
    async fn beacon(
        &mut self
    ) -> anyhow::Result<()>
    {
        // Predefine all info that does not have to be redefined later in the critical section.
        let beacon_interval: Duration;
        let identifier: Identifier;
        let payload: Payload;
        {
            let config = self.endpoint.config_mut();
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
                let config = self.endpoint.config_mut();
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
            self.tx_message(message).await?;

            // pause to avoid excess traffic
            interval.tick().await;
        }
    }

    // /// Used by the main thread to send stuff with multicast.
    // /// tx_beacon is a private method, so this is used to call tx_beacon whenever a
    // pub async fn run_multicast_transmitter(
    //     &mut self,
    //     mut rx_messages_for_transmission: tokio::sync::mpsc::Receiver<Message>,
    // ) -> anyhow::Result<()>
    // {
    //     // Loop and receive messages from the main thread while the channel is open.
    //     while let Some(message) = rx_messages_for_transmission.recv().await
    //     {
    //         // Send the message.
    //         self.tx_message(message).await?
    //     }
    //
    //     Ok(())
    // }

    /// Listens for incoming messages
    async fn run_multicast_listener(&mut self) -> anyhow::Result<()>
    {
        let (tx_received_messages, rx_received_messages) = mpsc::channel::<Message>(100);
        self.rx_incoming_messages = Some(rx_received_messages);

        let result = tokio::spawn(async move {
            // Keep receiving messages for as long as the program is running.
            while let Ok(message) = self.rx_message().await
            {
                tx_received_messages.send(message).await;
            }
        }).await?;

        Ok(())
    }

    /// Returns the handle for the stream of messages received by the multicast listener.
    async fn listener(&self) -> &Option<mpsc::Receiver<Message>>
    {
        &self.rx_incoming_messages
    }

    /// Receives and processes an incoming multicast datagram.
    ///
    /// Will attempt to parse the message and perform the appropriate action.
    async fn rx_message(
        &mut self,
    ) -> anyhow::Result<Message>
    {
        let (packet_type, message): (PacketType, Option<Message>);
        {
            if let Ok((received_packet_type, received_message)) = self.endpoint.rx().await {
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
        &mut self,
        message: Message,
    ) -> anyhow::Result<()>
    {
        // send IPv4 beacon if configured
        if !self.endpoint.config().m_addr4.is_empty()
        {
            if self.endpoint.tx(PacketType::Ipv4, &message.to_string().as_bytes()).await?
            {
                info!("tx4-> {}", &message);
            }
        }

        // send IPv6 beacon if configured
        if !self.endpoint.config().m_addr6.is_empty()
        {
            if self.endpoint.tx(PacketType::Ipv6, &message.to_string().as_bytes()).await?
            {
                info!("tx6-> {}", &message);
            }
        }

        Ok(())
    }
}

impl Drop for Beacon
{
    fn drop(&mut self)
    {
        // Ensure that we leave the multicast group before exiting.
        let _ = self.endpoint.leave();
    }
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
