//! shared utilities for TCP client and server

pub mod client;
pub mod server;

use anyhow::anyhow;
use std::net::{IpAddr, Ipv6Addr};
use log::info;

/// this implementation works for lab machines
pub fn get_link_local_ipv6() -> anyhow::Result<(Ipv6Addr, String)>
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
pub fn get_uid_port() -> u16
{
    use nix::unistd::Uid;

    let uid = Uid::current().as_raw();
    let port = uid as u16;

    port
}