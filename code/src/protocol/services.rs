use std::{
    fmt::{self, Formatter},
    marker::PhantomData,
};

/// This makes it harder to put the boolean values in the wrong way round.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SearchAvailability(pub bool);

impl fmt::Display for SearchAvailability
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DownloadAvailability(pub bool);

impl fmt::Display for DownloadAvailability
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Services
{
    pub search:   SearchAvailability,
    pub download: DownloadAvailability,
    // Force the new() constructor to be used.
    // https://rust-unofficial.github.io/too-many-lists/index.html
    _boo: PhantomData<usize>,
}

impl Services
{
    pub fn new(search: SearchAvailability, download: DownloadAvailability) -> Self
    {
        Self
        {
            search,
            download,
            _boo: Default::default(),
        }
    }
}

impl fmt::Display for Services
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        write!(f, "search={},download={}", self.search, self.download)
    }
}
