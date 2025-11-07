//! Multicast networking functionality wrapper using socket2
//! Rust version based on saleem's original Java

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::time::Duration;
use std::mem::MaybeUninit;
use socket2::{Socket, Domain, Type, Protocol, SockAddr};
use log::{info, error, debug};

use crate::configuration::Configuration;

/// type of packet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketType {
    None,
    Ipv4,
    Ipv6,
}

/// dual-stack multicast communication endpoint wrapper
pub struct MulticastEndpoint {
    socket_v4: Option<Socket>,
    socket_v6: Option<Socket>,
    mcast_addr_v4: Option<SocketAddr>,
    mcast_addr_v6: Option<SocketAddr>,
    config: Configuration,
}

impl MulticastEndpoint {
    /// Creates a new multicast endpoint.
    ///
    /// # Arguments
    /// * `config` - Application configuration
    pub fn new(config: &Configuration) -> Result<Self, Box<dyn std::error::Error>> {
        let mut endpoint = MulticastEndpoint {
            socket_v4: None,
            socket_v6: None,
            mcast_addr_v4: None,
            mcast_addr_v6: None,
            config: config.clone(),
        };
        
        // setup IPv4 socket if configured
        if !config.m_addr4.is_empty() && config.m_addr4 != "0" {
            endpoint.setup_ipv4()?;
        }
        
        // setup IPv6 socket if configured
        if !config.m_addr6.is_empty() && config.m_addr6 != "0" {
            endpoint.setup_ipv6()?;
        }
        
        if endpoint.socket_v4.is_none() && endpoint.socket_v6.is_none() {
            return Err("No multicast group defined".into());
        }
        
        Ok(endpoint)
    }
    
    /// sets up IPv4 multicast socket.
    fn setup_ipv4(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
        
        socket.set_reuse_address(self.config.reuse_addr)?;
        socket.set_multicast_loop_v4(self.config.loopback)?;
        socket.set_multicast_ttl_v4(self.config.m_ttl)?;
        socket.set_read_timeout(Some(Duration::from_millis(self.config.so_timeout)))?;
        
        let bind_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), self.config.m_port);
        socket.bind(&SockAddr::from(bind_addr))?;
        
        let mcast_addr: Ipv4Addr = self.config.m_addr4.parse()?;
        self.mcast_addr_v4 = Some(SocketAddr::new(IpAddr::V4(mcast_addr), self.config.m_port));
        
        socket.set_nonblocking(true)?;
        self.socket_v4 = Some(socket);
        
        debug!("IPv4 socket configured: {}", self.config.m_addr4);
        Ok(())
    }
    
    /// sets up IPv6 multicast socket.
    fn setup_ipv6(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let socket = Socket::new(Domain::IPV6, Type::DGRAM, Some(Protocol::UDP))?;
        
        socket.set_reuse_address(self.config.reuse_addr)?;
        socket.set_multicast_loop_v6(self.config.loopback)?;
        socket.set_multicast_hops_v6(self.config.m_ttl)?;
        socket.set_read_timeout(Some(Duration::from_millis(self.config.so_timeout)))?;
        
        let bind_addr = SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), self.config.m_port);
        socket.bind(&SockAddr::from(bind_addr))?;
        
        let mcast_addr: Ipv6Addr = self.config.m_addr6.parse()?;
        self.mcast_addr_v6 = Some(SocketAddr::new(IpAddr::V6(mcast_addr), self.config.m_port));
        
        socket.set_nonblocking(true)?;
        self.socket_v6 = Some(socket);
        
        debug!("IPv6 socket configured: {}", self.config.m_addr6);
        Ok(())
    }
    
    /// join the multicast group(s).
    pub fn join(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref socket) = self.socket_v4 {
            if let Some(addr) = self.mcast_addr_v4 {
                if let IpAddr::V4(mcast_addr) = addr.ip() {
                    socket.join_multicast_v4(&mcast_addr, &Ipv4Addr::UNSPECIFIED)?;
                    info!("Joined IPv4 multicast group {}", addr);
                }
            }
        }
        
        if let Some(ref socket) = self.socket_v6 {
            if let Some(addr) = self.mcast_addr_v6 {
                if let IpAddr::V6(mcast_addr) = addr.ip() {
                    socket.join_multicast_v6(&mcast_addr, 0)?;
                    info!("Joined IPv6 multicast group {}", addr);
                }
            }
        }
        
        Ok(())
    }
    
    /// leave the multicast group(s).
    pub fn leave(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref socket) = self.socket_v4 {
            if let Some(addr) = self.mcast_addr_v4 {
                if let IpAddr::V4(mcast_addr) = addr.ip() {
                    socket.leave_multicast_v4(&mcast_addr, &Ipv4Addr::UNSPECIFIED)?;
                    info!("Left IPv4 multicast group");
                }
            }
        }
        
        if let Some(ref socket) = self.socket_v6 {
            if let Some(addr) = self.mcast_addr_v6 {
                if let IpAddr::V6(mcast_addr) = addr.ip() {
                    socket.leave_multicast_v6(&mcast_addr, 0)?;
                    info!("Left IPv6 multicast group");
                }
            }
        }
        
        Ok(())
    }
    
    /// Receives a multicast packet.
    ///
    /// # Arguments
    /// * `buffer` - bufer to store received data
    ///
    /// # Returns
    /// the type of packet received (IPv4, IPv6, or None)
    pub fn rx(&mut self, buffer: &mut [u8]) -> Result<PacketType, Box<dyn std::error::Error>> {
        let mut uninit_buffer: Vec<MaybeUninit<u8>> = vec![MaybeUninit::uninit(); buffer.len()];
        
        // try IPv4
        if let Some(ref socket) = self.socket_v4 {
            match socket.recv(&mut uninit_buffer) {
                Ok(len) if len > 0 => {
                    // Copy initialized data to output buffer
                    for i in 0..len {
                        buffer[i] = unsafe { uninit_buffer[i].assume_init() };
                    }
                    debug!("Received {} bytes on IPv4", len);
                    return Ok(PacketType::Ipv4);
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // no data available, continue to IPv6
                }
                Err(e) => {
                    error!("IPv4 receive error: {}", e);
                }
                _ => {}
            }
        }
        
        // try IPv6
        if let Some(ref socket) = self.socket_v6 {
            match socket.recv(&mut uninit_buffer) {
                Ok(len) if len > 0 => {
                    // Copy initialized data to output buffer
                    for i in 0..len {
                        buffer[i] = unsafe { uninit_buffer[i].assume_init() };
                    }
                    debug!("Received {} bytes on IPv6", len);
                    return Ok(PacketType::Ipv6);
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // mo data available
                }
                Err(e) => {
                    error!("IPv6 receive error: {}", e);
                }
                _ => {}
            }
        }
        
        Ok(PacketType::None)
    }
    
    /// transmits a multicast packet.
    ///
    /// # Arguments
    /// * `pkt_type` - IPv4 or IPv6
    /// * `data` - Data to send
    ///
    /// # Returns
    /// `true` if successful, `false` otherwise
    pub fn tx(&mut self, pkt_type: PacketType, data: &[u8]) -> Result<bool, Box<dyn std::error::Error>> {
        match pkt_type {
            PacketType::Ipv4 => {
                if let (Some(ref socket), Some(addr)) = (&self.socket_v4, self.mcast_addr_v4) {
                    socket.send_to(data, &SockAddr::from(addr))?;
                    debug!("Sent {} bytes on IPv4", data.len());
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            PacketType::Ipv6 => {
                if let (Some(ref socket), Some(addr)) = (&self.socket_v6, self.mcast_addr_v6) {
                    socket.send_to(data, &SockAddr::from(addr))?;
                    debug!("Sent {} bytes on IPv6", data.len());
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            PacketType::None => Ok(false),
        }
    }
}
