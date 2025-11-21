//! Multicast networking functionality wrapper using socket2
//! Rust version based on saleem's original Java

use std::fmt;
use crate::{
    protocol::message::{Message},
    parser::message::parse_message,
    multicast::multicast_configuration::MulticastConfiguration,
};
use anyhow::{anyhow};
use log::{debug, error, info};
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    sync::Arc,
    time::Duration
};
use std::fmt::Formatter;
use bytes::BytesMut;
use tokio::{
    net::UdpSocket,
};

/// type of packet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketType
{
    None,
    Ipv4,
    Ipv6,
}

impl fmt::Display for PacketType
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        match self
        {
            PacketType::None => write!(f, "None"),
            PacketType::Ipv4 => write!(f, "IPv4"),
            PacketType::Ipv6 => write!(f, "IPv6"),
        }
    }
}

/// dual-stack multicast communication endpoint wrapper
#[derive(Clone, Debug)]
pub struct MulticastEndpoint
{
    socket_v4: Option<Arc<UdpSocket>>,
    socket_v6: Option<Arc<UdpSocket>>,
    pub mcast_addr_v4: Option<SocketAddr>,
    pub mcast_addr_v6: Option<SocketAddr>,
    pub(crate) config: MulticastConfiguration,
}

impl MulticastEndpoint
{
    /// Creates a new multicast endpoint.
    ///
    /// # Arguments
    /// * `config` - Application configuration
    pub fn new(config: MulticastConfiguration) -> anyhow::Result<Self>
    {
        info!("multicast config: {:#?}", config);

        let mut endpoint = Self {
            socket_v4: None,
            socket_v6: None,
            mcast_addr_v4: None,
            mcast_addr_v6: None,
            config: config.clone(),
        };

        // setup IPv4 socket if configured
        if !config.m_addr4.is_empty() && config.m_addr4 != "0"
        {
            endpoint.setup_ipv4()?;
        }

        // setup IPv6 socket if configured
        if !config.m_addr6.is_empty() && config.m_addr6 != "0"
        {
            endpoint.setup_ipv6()?;
            if let Some(socket) = &endpoint.socket_v6
            {
                if let Ok(link_local_addr) = socket.local_addr()
                {
                    if let SocketAddr::V6(mut addr) = link_local_addr
                    {
                        // addr.set_scope_id();
                        // let
                        // addr.
                        endpoint.config.ip_addr6 = Some(addr);
                        info!("set ipv6 address: {}", addr)
                    }
                }
            }
        }

        if endpoint.socket_v4.is_none() && endpoint.socket_v6.is_none()
        {
            return Err(anyhow!("No multicast group defined"));
        }

        Ok(endpoint)
    }

    /// sets up IPv4 multicast socket.
    fn setup_ipv4(&mut self) -> anyhow::Result<()>
    {
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

        // Convert to a tokio::net::UdpSocket
        let socket: std::net::UdpSocket = socket.into();
        self.socket_v4 = Some(Arc::new(UdpSocket::from_std(socket)?));

        debug!("IPv4 socket configured: {}", self.config.m_addr4);
        Ok(())
    }

    /// sets up IPv6 multicast socket.
    fn setup_ipv6(&mut self) -> anyhow::Result<()>
    {
        let socket = Socket::new(Domain::IPV6, Type::DGRAM, Some(Protocol::UDP))?;

        socket.set_reuse_address(self.config.reuse_addr)?;
        socket.set_multicast_loop_v6(self.config.loopback)?;
        socket.set_multicast_hops_v6(self.config.m_ttl)?;
        socket.set_read_timeout(Some(Duration::from_millis(self.config.so_timeout)))?;

        let bind_addr = SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), self.config.m_port);
        socket.bind(&SockAddr::from(bind_addr))?;

        info!("IPv6 socket configured: {}", self.config.m_addr6);

        let mcast_addr: Ipv6Addr = self.config.m_addr6.parse()?;
        self.mcast_addr_v6 = Some(SocketAddr::new(IpAddr::V6(mcast_addr), self.config.m_port));

        info!("IPv6 mcast addr v6 configured: {:#?}", self.mcast_addr_v6);

        socket.set_nonblocking(true)?;

        // Convert to a tokio::net::UdpSocket
        let socket: std::net::UdpSocket = socket.into();
        self.socket_v6 = Some(Arc::new(UdpSocket::from_std(socket)?));

        debug!("IPv6 socket configured: {}", self.config.m_addr6);
        Ok(())
    }

    /// join the multicast group(s).
    pub fn join(&mut self) -> anyhow::Result<()>
    {
        if let Some(ref socket) = self.socket_v4
        {
            if let Some(addr) = self.mcast_addr_v4
            {
                if let IpAddr::V4(mcast_addr) = addr.ip()
                {
                    socket.join_multicast_v4(mcast_addr, Ipv4Addr::UNSPECIFIED)?;
                    info!("Joined IPv4 multicast group {}", addr);
                }
            }
        }

        if let Some(ref socket) = self.socket_v6
        {
            if let Some(addr) = self.mcast_addr_v6
            {
                if let IpAddr::V6(mcast_addr) = addr.ip()
                {
                    socket.join_multicast_v6(&mcast_addr, 0)?;
                    info!("Joined IPv6 multicast group {}", addr);
                }
            }
        }

        Ok(())
    }

    /// leave the multicast group(s).
    pub fn leave(&mut self) -> anyhow::Result<()>
    {
        if let Some(ref socket) = self.socket_v4
        {
            if let Some(addr) = self.mcast_addr_v4
            {
                if let IpAddr::V4(mcast_addr) = addr.ip()
                {
                    socket.leave_multicast_v4(mcast_addr, Ipv4Addr::UNSPECIFIED)?;
                    info!("Left IPv4 multicast group");
                }
            }
        }

        if let Some(ref socket) = self.socket_v6
        {
            if let Some(addr) = self.mcast_addr_v6
            {
                if let IpAddr::V6(mcast_addr) = addr.ip()
                {
                    socket.leave_multicast_v6(&mcast_addr, 0)?;
                    info!("Left IPv6 multicast group");
                }
            }
        }

        Ok(())
    }

    pub async fn rx(&self) -> anyhow::Result<(PacketType, Message)>
    {
        info!("rx called");
        let mut buffer_1 = BytesMut::with_capacity(self.config.msg_size);
        let mut buffer_2 = BytesMut::with_capacity(self.config.msg_size);

        // Whichever packet is received first will be returned
        tokio::select! {
            packet_type = self.rx_ipv4(&mut buffer_1) => {
                info!("rx_ipv4 returned");
                info!("buffer contents: {:#?}", str::from_utf8(&buffer_1));
                Ok((packet_type?, parse_message(&buffer_1)?))
            },
            packet_type = self.rx_ipv6(&mut buffer_2) => {
                info!("rx_ipv6 returned");
                info!("buffer contents: {:#?}", str::from_utf8(&buffer_2));
                Ok((packet_type?, parse_message(&buffer_2)?))
            },
        }
    }

    /// Receives a multicast packet.
    ///
    /// # Arguments
    /// * `buffer` - buffer to store received data
    ///
    /// # Returns
    /// the type of packet received (IPv4, IPv6, or None)
    async fn rx_ipv4(&self, buffer: &mut BytesMut) -> anyhow::Result<PacketType>
    {
        info!("rx_ipv4 called");

        // try IPv4
        if let Some(socket) = &self.socket_v4
        {
            match socket.recv(buffer).await
            {
                Ok(len) if len > 0 => {
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

        Ok(PacketType::None)
    }

    async fn rx_ipv6(&self, buffer: &mut BytesMut) -> anyhow::Result<PacketType>
    {
        info!("rx_ipv6 called");

        // try IPv6
        if let Some(socket) = &self.socket_v6
        {
            match socket.recv(buffer).await
            {
                Ok(len) if len > 0 => {
                    debug!("Received {} bytes on IPv6", len);
                    return Ok(PacketType::Ipv6);
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    error!("IPv6 receive error: {}", e);
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
    pub async fn tx(
        &self,
        pkt_type: PacketType,
        data: &[u8],
    ) -> anyhow::Result<bool>
    {
        match pkt_type
        {
            PacketType::Ipv4 => {
                if let (Some(socket), Some(addr)) = (self.socket_v4.clone(), self.mcast_addr_v4)
                {
                    socket.send_to(data, addr).await?;
                    debug!("Sent {} bytes on IPv4, addr: {}", data.len(), addr);
                    debug!("socket local addr: {:#?}", socket.local_addr());
                    Ok(true)
                }
                else
                {
                    Ok(false)
                }
            }
            PacketType::Ipv6 => {
                if let (Some(socket), Some(addr)) = (self.socket_v6.clone(), self.mcast_addr_v6)
                {
                    socket.send_to(data, addr).await?;
                    debug!("Sent {} bytes on IPv6, addr: {}", data.len(), addr);
                    debug!("socket local addr: {:#?}", socket.local_addr());
                    Ok(true)
                }
                else
                {
                    Ok(false)
                }
            }
            PacketType::None => Ok(false),
        }
    }

    pub fn config(&self) -> &MulticastConfiguration
    {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut MulticastConfiguration
    {
        &mut self.config
    }
}

/// Leave the multicast group when this endpoint is dropped.
impl Drop for MulticastEndpoint
{
    fn drop(&mut self)
    {
        let _ = self.leave();
    }
}
