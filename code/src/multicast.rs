//! Multicast beacon application.
//! Rust version based on saleem's original Java

pub mod multicast_configuration;
pub mod multicast_endpoint;

use log::trace;
use std::net::SocketAddr;
use std::net::IpAddr;
use anyhow::anyhow;
use log::error;
use log::debug;
use log::info;
use crate::message_socket::MessageSocket;
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
    num::{NonZeroU64, NonZeroU16}
};

/// Send a beacon ping out every so often
pub async fn beacon(
    mut socket: MessageSocket,
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
        beacon_interval = Duration::from_millis(socket.endpoint().config.beacon_interval);
        identifier = Identifier::new(
            UID(socket.endpoint().config.username.clone()),
            FQDN(socket.endpoint().config.host_name.clone()),
        );
        payload = Payload::Advertisement(
            AdvertisementMessage::new(
                NonZeroU16::new(socket.endpoint().config.m_port)
                    .with_context(|| "beacon(): invalid port number")?,
                Services::new(
                    search_availability,
                    download_availability,
                )
            )
        );

        debug!("created defaults");
    }

    // The beacon will send an advertisement message once every [interval] milliseconds.
    let mut interval = tokio::time::interval(beacon_interval);
    debug!("tokio interval of {:#?} milliseconds", beacon_interval);
    loop
    {
        // Pause to avoid excess traffic.
        interval.tick().await;
        trace!("interval tick registered");

        trace!("entered beacon loop");

        // Get the next serial number from the configuration.
        let next_serial_number: NonZeroU64;
        next_serial_number = socket.endpoint_mut().config.get_next_serial_number();
        trace!("got serial number");

        // Header must be different each iteration because timestamp is constantly changing.
        let message = Message::new(
            Header::new(
                identifier.clone(),
                SerialNumber(next_serial_number.clone()),
                Timestamp::now()?,
            ),
            payload.clone()
        );
        trace!("created message");

        // Send the beacon message.
        socket.tx_message(message.clone()).await?;
        debug!("sent beacon {}; {}", next_serial_number, message);
    }
}
