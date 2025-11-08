use std::{
    fmt::{
        self,
        Formatter,
    },
    num::NonZeroU16,
};

#[derive(Clone, Debug)]
pub struct AdvertisementMessage
{
    server_port: NonZeroU16,
    services:    Services,
}

impl fmt::Display for AdvertisementMessage
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        write!(f, "advertisement:{}:{}", self.server_port, self.services)
    }
}


#[derive(Clone, Debug)]
pub struct Services
{
    search:   bool,
    download: bool,
}

impl fmt::Display for Services
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        write!(f, "search={},download={}", self.search, self.download)
    }
}

