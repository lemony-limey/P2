//! Multicast beacon application.
//! Rust version based on saleem's original Java

mod configuration;
mod multicast;

use log::info;
use std::thread;
use std::time::Duration;

use configuration::Configuration;
use multicast::{MulticastEndpoint, PacketType};

const TX_RATIO: u32 = 5;

fn main() -> Result<(), Box<dyn std::error::Error>>
{
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();

    info!("Starting Beacon application");

    let config = Configuration::load("beacon.properties")?;
    let username = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "user".to_string());

    let mut endpoint = MulticastEndpoint::new(&config)?;
    endpoint.join()?;

    for b in (1..=config.beacon).rev()
    {
        // transmit a beacon periodically
        if b % TX_RATIO == 0
        {
            tx_beacon(&mut endpoint, &config, &username)?;
        }

        // check for incoming beacons
        rx_beacon(&mut endpoint, &config)?;

        // pause to avoid excess traffic
        thread::sleep(Duration::from_millis(config.sleep_time));
    }

    endpoint.leave()?;
    info!("Beacon application terminated");

    Ok(())
}

/// receives and processes an incoming beacon.
fn rx_beacon(
    endpoint: &mut MulticastEndpoint,
    config: &Configuration,
) -> Result<(), Box<dyn std::error::Error>>
{
    let mut buffer = vec![0u8; config.msg_size];

    match endpoint.rx(&mut buffer)?
    {
        PacketType::None => Ok(()),
        PacketType::Ipv4 => {
            let msg = String::from_utf8_lossy(&buffer)
                .trim_end_matches('\0')
                .trim()
                .to_string();
            info!("->rx4 {}", msg);
            Ok(())
        }
        PacketType::Ipv6 => {
            let msg = String::from_utf8_lossy(&buffer)
                .trim_end_matches('\0')
                .trim()
                .to_string();
            info!("->rx6 {}", msg);
            Ok(())
        }
    }
}

/// transmits a beacon message.
fn tx_beacon(
    endpoint: &mut MulticastEndpoint,
    config: &Configuration,
    username: &str,
) -> Result<(), Box<dyn std::error::Error>>
{
    let now = chrono::Local::now().format("%Y%m%d-%H%M%S%.3f");
    let body = format!("|{}|{}|{}|", now, username, config.host_name);

    // send IPv4 beacon if configured
    if !config.m_addr4.is_empty()
    {
        let msg = format!("{}{}|", body, config.ip_addr4);
        let bytes = msg.as_bytes();

        if endpoint.tx(PacketType::Ipv4, bytes)?
        {
            info!("tx4-> {}", msg);
        }
    }

    // send IPv6 beacon if configured
    if !config.m_addr6.is_empty()
    {
        let msg = format!("{}{}|", body, config.ip_addr6);
        let bytes = msg.as_bytes();

        if endpoint.tx(PacketType::Ipv6, bytes)?
        {
            info!("tx6-> {}", msg);
        }
    }

    Ok(())
}
