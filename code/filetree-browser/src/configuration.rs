use std::num::NonZeroU64;
use std::marker::PhantomData;
use serde::Deserialize;
use crate::serial_number::SerialNumberGenerator;

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct Configuration
{
    pub log_file: String,
    pub uid: String,
    pub fqdn: String,
    pub root_dir: String,
    pub m_addr6: String,
    pub m_port: u16,
    pub m_ttl: u8,
    pub loopback: bool,
    pub reuse_addr: bool,
    pub so_timeout: u64,
    pub sleep_time: u64,
    pub maximum_message_size: usize,
    pub maximum_advertisement_period: u64,
    pub search: String,
    pub download: bool,
    // Do not attempt to serialise or deserialise the following.
    #[serde(skip)]
    serial_number_generator: SerialNumberGenerator,
}

impl Default for Configuration 
{
    fn default() -> Self 
    {
        let hostname = hostname::get()
            .ok()
            .and_then(|h| h.into_string().ok())
            .unwrap_or_else(|| "FileTreeBrowser-host".to_string());

        let username = std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .unwrap_or_else(|_| "user".to_string());

        eprintln!("hostname: {}, username: {}", hostname, username);

        Self 
        {
            log_file: "filetreebrowser.log".to_string(),
            uid: username,
            fqdn: hostname,
            root_dir: "root_dir".to_string(),
            m_addr6: "ff02::4105:4105".to_string(),
            m_port: 4105,
            m_ttl: 2,
            loopback: true,
            reuse_addr: false,
            so_timeout: 1,
            sleep_time: 5000,
            maximum_message_size: 500,
            maximum_advertisement_period: 1000,
            search: "none".to_string(),
            download: false,
            serial_number_generator: Default::default(),
        }
    }
}

impl Configuration 
{
    pub fn load(properties_file: &str) -> anyhow::Result<Self>
    {
        let settings = config::Config::builder()
            .add_source(config::File::with_name(properties_file).required(false))
            .build()?;

        println!("{:#?}", settings);

        let mut config = settings.try_deserialize().unwrap_or_else(|_| {
            log::error!("Using default configuration");
            Configuration::default()
        });

        println!("{:#?}", config);

        // update log file with id if using default
        if config.log_file == "filetreebrowser.log"
        {
            config.log_file = format!("{}-log.log", config.uid);
        }

        log::info!("Configuration loaded from {}", properties_file);
        log::debug!("Config details: {:?}", config);

        Ok(config)
    }

    pub fn get_next_serial_number(&mut self) -> NonZeroU64
    {
        self.serial_number_generator.get_next_serial_number()
    }
}
