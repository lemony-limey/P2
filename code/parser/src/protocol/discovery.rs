use std::{
    fmt::{self, Formatter},
    num::NonZeroU16,
};

#[derive(Clone, Debug)]
pub struct AdvertisementMessage
{
    pub server_port: NonZeroU16,
    pub services:    Services,
}

impl fmt::Display for AdvertisementMessage
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        write!(f, "advertisement:{}:{}", self.server_port, self.services)
    }
}


#[derive(Clone, Debug, Default)]
pub struct Services
{
    pub search:   bool,
    pub download: bool,
}

impl fmt::Display for Services
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        write!(f, "search={},download={}", self.search, self.download)
    }
}
