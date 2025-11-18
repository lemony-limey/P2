use std::path::PathBuf;
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
use std::hash::{Hash, Hasher};
use std::net::Ipv6Addr;
use std::num::NonZeroU16;
use std::ops::Add;
use std::path::Path;
use anyhow::Context;
use chrono::{DateTime, TimeDelta, Utc};
use tokio::sync::mpsc;
use crate::message_socket::MessageSocket;


#[derive(Clone, Debug)]
pub struct FTBSoftState
{
    nodes: HashSet<Arc<RwLock<Node>>>,
    // files: HashSet<PathBuf>,
}

impl FTBSoftState
{
    pub fn new() -> Self
    {
        Self
        {
            nodes: HashSet::new(),
        }
    }

    /// Return the handle to the first node found that contains the specified file.
    pub fn find_file(&self, target: &str) -> Option<Arc<RwLock<Node>>>
    {

        todo!()
    }


    pub fn print_all_nodes(&self)
    {
        if self.nodes.is_empty()
        {
            println!("No other nodes on the network currently.");
            return;
        }

        for node in &self.nodes
        {
            println!("{}|{}", node.identifier, node.ipv6addr);
        }
    }

    pub fn discovery_update(&mut self, node: Node)
    {
        if !self.nodes.contains(&node)
        {
            self.nodes.insert(node);
            return;
        }

        // If nodes does contain this node already, update the fields and refresh the ttl value.

    }

    /// Return
    pub fn contains_file(file_name: &str) -> Option<Arc<RwLock<Node>>>
    {

        None
    }
}


/// This is where we store info about nodes discovered in the network.
#[derive(Clone, Debug)]
pub struct Node
{
    pub identifier: Identifier,
    pub ipv6addr: Ipv6Addr,
    pub server_port: NonZeroU16,
    pub services: Services,
    pub files: HashSet<FileRecord>,
    pub ttl: DateTime<Utc>,
}

impl PartialEq for Node
{
    fn eq(&self, other: &Self) -> bool
    {
        self.identifier.eq(&other.identifier) // && self.ipv6addr.eq(&other.ipv6addr)
    }
}

impl Eq for Node {}

/// This allows us to use the "contains()" method on a HashMap containing Nodes.
impl Hash for Node
{
    fn hash<H: Hasher>(&self, state: &mut H)
    {
        self.identifier.hash(state);
    }
}

impl Node
{
    pub fn new(
        identifier: Identifier,
        ipv6addr: Ipv6Addr,
        server_port: NonZeroU16,
        services: Services,
    ) -> anyhow::Result<Self>
    {
        Ok(Self
        {
            identifier,
            ipv6addr,
            server_port,
            services,
            files: HashSet::new(),
            ttl: Utc::now().add(
                TimeDelta::new(30, 0)
                .with_context(|| "Node::new(): failed to add ttl value")?
            ),
        }
        )
    }
}


#[derive(Clone, Debug)]
pub struct FileRecord
{
    file_name: PathBuf,
    ttl: Utc,
}
