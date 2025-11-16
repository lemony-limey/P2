use crate::{
    protocol::{
        message::Message,
        identifier::Identifier,
        message::Timestamp,
        services::Services
    }
};
use std::collections::HashSet;
use std::num::NonZeroU16;
use tokio::sync::mpsc;

pub struct SoftState
{
    nodes: HashSet<Node>,
    rx_incoming_messages: mpsc::Receiver<Message>,
}

/// This is where we store info about nodes discovered in the network.
/// TODO: Implement hasher for node
pub struct Node
{
    identifier: Identifier,
    first_discovered: Timestamp,
    server_port: NonZeroU16,
    services: Services,
}
