use std::collections::HashSet;
use std::num::NonZeroU16;
use parser::protocol::identifier::Identifier;
use parser::protocol::message::Timestamp;
use parser::protocol::services::Services;

pub struct SoftState
{
    nodes: HashSet<Node>
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