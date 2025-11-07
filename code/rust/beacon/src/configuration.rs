//! Configuration handling for the beacon application.
//! Rust version based on saleem's original Java version

use serde::Deserialize;
use std::net::IpAddr;

/// Beacon application configuration.
#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct Configuration {
    pub log_file: String,
    pub m_addr4: String,
    pub m_addr6: String,
    pub m_port: u16,
    pub m_ttl: u32,
    pub loopback: bool,
    pub reuse_addr: bool,
    pub so_timeout: u64,
    pub msg_size: usize,
    pub sleep_time: u64,
    pub beacon: u32,
    
    // detected at run-time
    #[serde(skip)]
    pub host_name: String,
    #[serde(skip)]
    pub ip_addr4: String,
    #[serde(skip)]
    pub ip_addr6: String,
}

impl Default for Configuration {
    fn default() -> Self {
        Configuration {
            log_file: "beacon.log".to_string(),
            m_addr4: "239.0.98.76".to_string(),
            m_addr6: "ff02::9876:9876".to_string(),
            m_port: 9876,
            m_ttl: 2,
            loopback: false,
            reuse_addr: true,
            so_timeout: 1,
            msg_size: 128,
            sleep_time: 10000,
            beacon: 10,
            host_name: String::new(),
            ip_addr4: String::new(),
            ip_addr6: String::new(),
        }
    }
}

impl Configuration {
    /// automatically grab the local hostname and IP addresses
    pub fn load(properties_file: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let settings = config::Config::builder()
            .add_source(config::File::with_name(properties_file).required(false))
            .build()?;
        
        let mut config: Configuration = settings.try_deserialize()
            .unwrap_or_else(|_| {
                log::info!("Using default configuration");
                Configuration::default()
            });

        config.detect_network_info()?;
        
        log::info!("Configuration loaded from {}", properties_file);
        log::info!("Detected: {} {} {}", config.host_name, config.ip_addr4, config.ip_addr6);
        log::debug!("Config details: {:?}", config);
        
        Ok(config)
    }
    
    fn detect_network_info(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // grab hostname
        self.host_name = hostname::get()?
            .into_string()
            .unwrap_or_else(|_| "unknown".to_string());
        
        // grab local IPv4 address
        if let Ok(ip) = local_ip_address::local_ip() {
            match ip {
                IpAddr::V4(addr) => self.ip_addr4 = addr.to_string(),
                IpAddr::V6(addr) => self.ip_addr6 = addr.to_string(),
            }
        }
        
        // grab liink-local IPv6 address
        if let Ok(ip) = local_ip_address::local_ipv6() {
            self.ip_addr6 = ip.to_string();
        }
        
        Ok(())
    }
}
