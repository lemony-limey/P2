//! Multicast beacon application.
//! Rust version based on saleem's original Java

mod configuration;
mod multicast;

use configuration::Configuration;
use log::info;
use multicast::{MulticastEndpoint, PacketType};
use std::{
    sync::mpsc::{Receiver, SyncSender},
    thread,
    time::Duration
};


const TX_RATIO: u32 = 5;

fn run(
    tx_to_filetree_browser: &mut SyncSender<String>,
    rx_shutdown: &mut Receiver<bool>,
) -> anyhow::Result<()>
{
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();

    info!("Starting Beacon application");

    let config = Configuration::load("beacon.toml")?;
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
        rx_beacon(&mut endpoint, &config, tx_to_filetree_browser)?;

        // pause to avoid excess traffic
        thread::sleep(Duration::from_millis(config.sleep_time));

        // Check if shutdown has been signalled.
        if let Ok(_) = rx_shutdown.try_recv() { break; }
    }

    endpoint.leave()?;
    info!("Beacon application terminated");

    Ok(())
}

/// receives and processes an incoming beacon.
fn rx_beacon(
    endpoint: &mut MulticastEndpoint,
    config: &Configuration,
    tx_to_user: &mut SyncSender<String>, // Send the received messages to filetree-browser
) -> anyhow::Result<()>
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
            info!("->rx4 {}", &msg);
            tx_to_user.send(msg)?;
            Ok(())
        }
        PacketType::Ipv6 => {
            let msg = String::from_utf8_lossy(&buffer)
                .trim_end_matches('\0')
                .trim()
                .to_string();
            info!("->rx6 {}", &msg);
            tx_to_user.send(msg)?;
            Ok(())
        }
    }
}

/// transmits a beacon message.
fn tx_beacon(
    endpoint: &mut MulticastEndpoint,
    config: &Configuration,
    username: &str,
) -> anyhow::Result<()>
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
