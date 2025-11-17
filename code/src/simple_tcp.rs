//! shared utilities for TCP client and server

use std::num::NonZeroU16;
use std::str::FromStr;
pub mod client;
pub mod server;

use std::ffi::CString;
use std::io;
use anyhow::anyhow;
use std::net::{IpAddr, Ipv6Addr, SocketAddr, SocketAddrV6};
use log::info;

/// this implementation works for lab machines
pub(crate) fn get_link_local_ipv6() -> anyhow::Result<(Ipv6Addr, String)>
{
    for iface in local_ip_address::list_afinet_netifas()?
    {
        if let IpAddr::V6(addr) = iface.1
        {
            if addr.is_unicast_link_local()
            {
                info!("Found link-local IPv6: {} on interface {}", addr, iface.0);
                return Ok((addr, iface.0));
            }
        }
    }

    Err(anyhow!("No link-local IPv6 address found"))
}

/// gets the Unix UID for the current user
pub(crate) fn get_uid_port() -> Option<NonZeroU16>
{
    use nix::unistd::Uid;

    let uid = Uid::current().as_raw();
    let port = NonZeroU16::new(uid as u16);

    port
}


pub fn parse_ipv6_with_scope(addr_str: &str, port: NonZeroU16) -> anyhow::Result<SocketAddr>
{
    let port = port.get();

    // this is a little.. messy
    if let Some(percent_idx) = addr_str.find('%')
    {
        let ip_part = &addr_str[..percent_idx];
        let ifname = &addr_str[percent_idx + 1..];

        let ip = Ipv6Addr::from_str(ip_part)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

        // Convert interface name (e.g. "eno1") to index using libc
        let c_ifname = CString::new(ifname)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "bad iface name"))?;
        let ifindex = unsafe { libc::if_nametoindex(c_ifname.as_ptr()) };
        if ifindex == 0
        {
            return Err(anyhow!(
                io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("interface '{}' not found", ifname),
                )
            ));
        }

        Ok(SocketAddr::V6(SocketAddrV6::new(ip, port, 0, ifindex)))
    }
    else
    {
        // try parsing as a raw SocketAddr (e.g. "::1" or "127.0.0.1")
        if let Ok(ipv6) = Ipv6Addr::from_str(addr_str)
        {
            Ok(SocketAddr::V6(SocketAddrV6::new(ipv6, port, 0, 0)))
        }
        else
        {
            Err(anyhow!(
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("could not get IPv6 address from input string: \"{addr_str}\"")
                )
            ))
        }
    }
}
