use crate::protocol::services::Services;
use std::{
    fmt::{self, Formatter},
    num::NonZeroU16,
};
use std::marker::PhantomData;
use std::net::Ipv6Addr;

#[derive(Clone, Debug)]
pub struct AdvertisementMessage
{
    pub ipv6_link_local_addr: Ipv6Addr,
    pub server_port: NonZeroU16,
    pub services:    Services,
    _boo: PhantomData<usize>
}

impl AdvertisementMessage
{
    pub fn new(
        ipv6_link_local_addr: Ipv6Addr,
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
        write!(f, "advertisement:{}:{}", self.server_port, self.services)
    }
}
