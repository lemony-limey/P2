use std::collections::BTreeSet;
use std::net::SocketAddr;
use tokio::sync::RwLock;
use std::sync::Arc;
use crate::{
    protocol::{
        message::Message,
        identifier::Identifier,
        message::Timestamp,
        services::Services
    }
};
use std::collections::HashSet;
use std::net::Ipv6Addr;
use std::num::NonZeroU16;
use std::path::Path;
use tokio::sync::mpsc;
use crate::message_socket::MessageSocket;

#[derive(Clone, Debug)]
pub struct SoftState
{
    nodes: BTreeSet<Node>,
}

/// This is where we store info about nodes discovered in the network.
#[derive(Clone, Debug)]
pub struct Node
{
    pub identifier: Identifier,
    pub ipv6addr: SocketAddr,
    pub first_discovered: Timestamp,
    pub server_port: NonZeroU16,
    pub services: Services,
    pub files: HashSet<String>,
}

// TODO: Should IPv6 Addr be part of the comparison?
impl PartialEq for Node
{
    fn eq(&self, other: &Self) -> bool
    {
        self.identifier.eq(&other.identifier) // && self.ipv6addr.eq(&other.ipv6addr)
    }
}

impl Eq for Node {}

impl SoftState
{
    pub fn new() -> Self
    {
        Self
        {
            nodes: HashSet::new(),
        }
    }

    /// Return the handle to the first node found that contains the specified file.
    pub fn find_file(&self, target: &str) -> Option<&Node>
    {

        todo!()
    }
}
