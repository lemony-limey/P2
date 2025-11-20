//! Multicast beacon application.
//! Rust version based on saleem's original Java

pub mod multicast_configuration;
pub mod multicast_endpoint;

use log::debug;
use log::info;
use crate::message_socket::MessageSocket;
use tokio::sync::RwLock;
use anyhow::{Context};
use crate::{
    protocol::{
        discovery::AdvertisementMessage,
        identifier::{UID, FQDN, Identifier},
        message::{Header, Timestamp, Payload, Message},
        serial_number::SerialNumber,
        services::{DownloadAvailability, SearchAvailability, Services},
    },
};
use std::{
    time::Duration,
    sync::{Arc},
    num::{NonZeroU64, NonZeroU16}
};
use std::net::Ipv6Addr;


/// Send a beacon ping out every so often
pub async fn beacon(
    socket: Arc<RwLock<MessageSocket>>,
    search_availability: SearchAvailability,
    download_availability: DownloadAvailability,
) -> anyhow::Result<()>
{
    info!("started beacon");

    // Predefine all info that does not have to be redefined later in the critical section.
    let beacon_interval: Duration;
    let identifier: Identifier;
    let payload: Payload;
    {
        let socket_handle = socket.read().await;
        let endpoint = socket_handle.endpoint();
        let ipv6_addr_string = &socket_handle.endpoint().config.ip_addr6;
        debug!("ipv6_addr_string: {}", ipv6_addr_string);
        let ipv6_addr = ipv6_addr_string.parse::<Ipv6Addr>()
            .with_context(|| "attempting to parse the multicast config ipv6 address")?;
        beacon_interval = Duration::from_millis(endpoint.config.beacon_interval);
        identifier = Identifier::new(
            UID(endpoint.config.username.clone()),
            FQDN(endpoint.config.host_name.clone()),
        );
        payload = Payload::Advertisement(
            AdvertisementMessage::new(
                ipv6_addr,
                NonZeroU16::new(endpoint.config.m_port)
                    .with_context(|| "beacon(): invalid port number")?,
                Services::new(
                    search_availability,
                    download_availability,
                )
            )
        );

        info!("created default stuff");
    }

    // The beacon will send an advertisement message once every [interval] milliseconds.
    let mut interval = tokio::time::interval(beacon_interval);
    info!("tokio interval of {:#?} milliseconds", beacon_interval);
    loop
    {
        info!("entered beacon loop");
        // Get the next serial number from the configuration.
        let next_serial_number: NonZeroU64;
        {
            next_serial_number = socket.write().await.endpoint_mut().config.get_next_serial_number();
            info!("got serial number");
        }

        // Header must be different each iteration because timestamp is constantly changing.
        let message = Message::new(
            Header::new(
                identifier.clone(),
                SerialNumber(next_serial_number.clone()),
                Timestamp::now()?,
            ),
            payload.clone()
        );
        info!("created message");

        // Send the beacon message.
        {
            socket.read().await.tx_message(message).await?;
            info!("sent beacon {}", next_serial_number);
        }

        // Pause to avoid excess traffic.
        interval.tick().await;
        info!("interval tick registered");
    }
}
