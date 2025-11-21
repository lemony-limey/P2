use crate::protocol::services::Services;
use std::{
    fmt::{self, Formatter},
    num::NonZeroU16,
};
use std::marker::PhantomData;
use std::net::{Ipv6Addr, SocketAddrV6};

#[derive(Clone, Debug, PartialEq)]
pub struct AdvertisementMessage
{
    pub ipv6_link_local_addr: SocketAddrV6,
    pub server_port: NonZeroU16,
    pub services:    Services,
    // Force the new() constructor to be used.
    // https://rust-unofficial.github.io/too-many-lists/index.html
    _boo: PhantomData<usize>
}

impl AdvertisementMessage
{
    pub fn new(
        ipv6_link_local_addr: SocketAddrV6,
        server_port: NonZeroU16,
        services: Services
    ) -> Self
    {
        Self
        {
            ipv6_link_local_addr,
            server_port,
            services,
            _boo: Default::default()
        }
    }
}

impl fmt::Display for AdvertisementMessage
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        write!(
            f,
            "advertisement:[{}%{}]:{}:{}",
            self.ipv6_link_local_addr.ip(),
            self.ipv6_link_local_addr.scope_id(),
            self.server_port,
            self.services,
        )
    }
}
