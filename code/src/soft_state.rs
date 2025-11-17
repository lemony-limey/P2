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
use tokio::sync::mpsc;
use crate::message_socket::MessageSocket;

#[derive(Clone, Debug)]
pub struct SoftState
{
    nodes: HashSet<Node>,
    socket: MessageSocket,
}

/// This is where we store info about nodes discovered in the network.
/// TODO: Implement hasher for node
#[derive(Clone, Debug)]
pub struct Node
{
    identifier: Identifier,
    ipv6addr: Ipv6Addr,
    first_discovered: Timestamp,
    server_port: NonZeroU16,
    services: Services,
    files: Vec<String>,
}
