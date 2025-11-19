use std::collections::HashMap;
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
use tokio::sync::mpsc::Receiver;
use crate::logical_file_path::LogicalFilePath;
use crate::message_socket::MessageSocket;


#[derive(Clone, Debug)]
pub struct FTBSoftState
{
    pub nodes: Arc<RwLock<HashMap<Identifier, Arc<RwLock<Node>>>>>,
}

impl FTBSoftState
{
    pub fn new() -> Self
    {
        Self
        {
            nodes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Return the handle to any node found that contains the specified file.
    /// Exact matches only.
    pub async fn find_file(&self, target: &LogicalFilePath) -> Option<HashSet<Identifier>>
    {
        let mut results = HashSet::new();
        let nodes_handle = self.nodes.read().await;

        // Iterate over every node and save the identifiers of any node with the target file
        for (identifier, node) in nodes_handle.iter()
        {
            if node.read().await.files.contains_key(target)
            {
                results.insert(identifier.clone());
            }
        }

        if results.is_empty()
        {
            None
        }
        else
        {
            Some(results)
        }
    }


    pub async fn print_all_nodes(&self)
    {
        if self.nodes.read().await.is_empty()
        {
            println!("No other nodes on the network currently.");
            return;
        }

        for (identifier, node) in self.nodes.read().await.iter()
        {
            let node = node.read().await;
            println!("{}|{}", identifier, node.address.ip());
        }
    }

    pub async fn discovery_update(&mut self, node: Node) -> anyhow::Result<()>
    {
        // If the node is inserted successfully and None is returned, we know there was no node
        // with the same identifier in the HashMap before.
        {
            if self.nodes.write().await.insert(
                node.identifier.clone(),
                Arc::new(RwLock::new(node)),
            ).is_none() { return Ok(()); }
        }

        // If nodes does contain this node already, update the fields and refresh the ttl value.
        Ok(())
    }

    /// Check whether a given node contains a specific file
    pub async fn contains_file(&self, identifier: &Identifier, target_file: &LogicalFilePath) -> bool
    {
        // If the node corresponding to the specified identifier contains the file listed, return
        // true.
        if let Some(node) = self.nodes.read().await.get(identifier)
        {
            let mut node_handle = node.write().await;
            if let Some(ttl) = node_handle.files.get(target_file)
            {
                // If the entry is still valid (i.e., it expires at a later time), return true.
                if *ttl > Utc::now()
                {
                    return true;
                }
                // Remove invalid entry
                else
                {
                    node_handle.files.remove(target_file);
                    return false;
                }
            }
        }

        false
    }
}


/// This is where we store info about nodes discovered in the network.
#[derive(Clone, Debug)]
pub struct Node
{
    pub identifier: Identifier,
    pub address: SocketAddr,
    pub services: Services,
    pub files: HashMap<LogicalFilePath, DateTime<Utc>>,
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
        address: SocketAddr,
        services: Services,
    ) -> anyhow::Result<Self>
    {
        Ok(Self
        {
            identifier,
            address,
            services,
            files: HashMap::new(),
            ttl: Utc::now().add(
                TimeDelta::new(30, 0)
                .with_context(|| "Node::new(): failed to add ttl value")?
            ),
        }
        )
    }
}
