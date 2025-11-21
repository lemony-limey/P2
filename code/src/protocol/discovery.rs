use crate::protocol::services::Services;
use std::{
    fmt::{self, Formatter},
    num::NonZeroU16,
};
use std::marker::PhantomData;

#[derive(Clone, Debug, PartialEq)]
pub struct AdvertisementMessage
{
    pub server_port: NonZeroU16,
    pub services:    Services,
    // Force the new() constructor to be used.
    // https://rust-unofficial.github.io/too-many-lists/index.html
    _boo: PhantomData<usize>
}

impl AdvertisementMessage
{
    pub fn new(
        server_port: NonZeroU16,
        services: Services
    ) -> Self
    {
        Self
        {
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
            "advertisement:{}:{}",
            self.server_port,
            self.services,
        )
    }
}
