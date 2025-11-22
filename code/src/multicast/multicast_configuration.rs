//! Configuration handling for the beacon application.
//! Rust version based on saleem's original Java version

use log::debug;
use log::info;
use std::num::NonZeroU64;
use serde::Deserialize;
use std::net::{IpAddr, Ipv4Addr, SocketAddrV6};
use anyhow::Context;
use crate::protocol::serial_number::SerialNumberGenerator;

/// Beacon application configuration.
#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct MulticastConfiguration
{
    pub log_file: String,
    pub m_addr4: String,
    pub m_addr6: String,
    pub m_port: u16,
    pub m_ttl: u32,
    pub loopback: bool,
    pub reuse_addr: bool,
    pub so_timeout: u64,
    pub msg_size: usize,
    pub beacon_interval: u64,
    pub beacon: u32,

    // detected at run-time
    #[serde(skip)]
    pub host_name: String,
    #[serde(skip)]
    pub ip_addr4: Option<Ipv4Addr>,
    #[serde(skip)]
    pub ip_addr6: Option<SocketAddrV6>,
    #[serde(skip)]
    pub username: String,
    #[serde(skip)]
    serial_number_generator: SerialNumberGenerator,
}

impl Default for MulticastConfiguration
{
    fn default() -> Self
    {
        Self
        {
            log_file: "beacon.log".to_string(),
            m_addr4: "239.0.98.76".to_string(),
            m_addr6: "ff02::2:3756".to_string(),
            m_port: 23756,
            m_ttl: 2,
            loopback: false,
            reuse_addr: true,
            so_timeout: 1,
            msg_size: 128,
            beacon_interval: 5000,  // 5s
            beacon: 10,
            host_name: String::new(),
            ip_addr4: None,
            ip_addr6: None,
            username: String::new(),
            serial_number_generator: Default::default(),
        }
    }
}

impl MulticastConfiguration
{
    /// automatically grab the local hostname and IP addresses
    pub fn load(properties_file: &str) -> anyhow::Result<Self>
    {
        let settings = config::Config::builder()
            .add_source(config::File::with_name(properties_file).required(false))
            .build()?;

        let mut config: MulticastConfiguration = settings.try_deserialize().unwrap_or_else(|_| {
            info!("Using default configuration");
            MulticastConfiguration::default()
        });

        // This sets the values of ip_addr4 and ip_addr6
        config.detect_network_info()
            .with_context(|| "couldn't call detect network info".to_string())?;

        info!("Configuration loaded from {}", properties_file);
        info!(
            "Detected: {:#?} {:#?} {:#?}",
            config.host_name,
            config.m_addr4,
            config.m_addr6
        );
        debug!("Config details: {:?}", config);

        info!(
            "Detected: {:#?} {:#?} {:#?}",
            config.host_name,
            config.ip_addr4,
            config.ip_addr6,
        );

        Ok(config)
    }

    fn detect_network_info(&mut self) -> anyhow::Result<()>
    {
        // grab hostname
        self.host_name = hostname::get()
            .with_context(|| "could not get host name")?
            .into_string()
            .unwrap_or_else(|_| "unknown".to_string());

        // grab local IPv4 address
        if let Ok(IpAddr::V4(ip)) = local_ip_address::local_ip()
        {
            info!("ipv4: {}", ip);
            self.ip_addr4 = Some(ip);
        }

        // N.B, the local_ip_address::local_ipv6() function did not appear to
        // work on the linux lab machines during testing, so the IPv6 address
        // is set using the local value of the UDP socket for multicast instead.
        // grab link-local IPv6 address
        // let local_ipv6_result = local_ip_address::local_ipv6();
        // if let Ok(IpAddr::V6(ip)) = local_ipv6_result
        // {
        //     info!("ipv6: {}", ip);
        //     self.ip_addr6 = Some(ip);
        // }
        // else {
        //     return Err(anyhow!(local_ipv6_result?))
        // }

        Ok(())
    }

    pub fn get_next_serial_number(&mut self) -> NonZeroU64
    {
        self.serial_number_generator.get_next_serial_number()
    }
}
