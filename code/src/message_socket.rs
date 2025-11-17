use anyhow::anyhow;
use log::info;
use crate::beacon::multicast::{MulticastEndpoint, PacketType};
use crate::protocol::message::Message;

/// MessageSocket is an abstraction over MulticastEndpoint.
/// While MulticastEndpoint is responsible for sending and receiving UDP datagrams, it is the job
/// of MessageSocket to provide an interface over this for sending objects of the Message type.
/// This means that MulticastEndpoint can be used
#[derive(Clone, Debug)]
pub struct MessageSocket
{
    endpoint: MulticastEndpoint,
}

impl MessageSocket
{
    pub fn new(endpoint: MulticastEndpoint) -> Self
    {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
            .format_timestamp_millis()
            .init();
        info!("Starting Beacon application");

        Self { endpoint }
    }

    pub fn endpoint(&self) -> &MulticastEndpoint
    {
        &self.endpoint
    }

    pub fn endpoint_mut(&mut self) -> &mut MulticastEndpoint
    {
        &mut self.endpoint
    }

    /// Receives and processes an incoming multicast datagram.
    ///
    /// Will attempt to parse the message and perform the appropriate action.
    pub async fn rx_message(
        &self,
    ) -> anyhow::Result<Message>
    {
        let (packet_type, message): (PacketType, Option<Message>);
        {
            if let Ok((received_packet_type, received_message)) = self.endpoint.rx().await {
                packet_type = received_packet_type;
                message = Some(received_message);
            }
            else
            {
                packet_type = PacketType::None;
                message = None;
            }
        }

        match (packet_type, message)
        {
            (PacketType::None, _) => Err(anyhow!("an error occurred when reading the packet")),
            (PacketType::Ipv4, Some(message)) => {
                info!("->rx4 {}", &message);
                Ok(message)
            },
            (PacketType::Ipv6, Some(message)) => {
                info!("->rx6 {}", &message);
                Ok(message)
            },
            (_, _) => {
                Err(anyhow!("rx_message(): invalid combination of message and packet type"))
            }
        }
    }

    /// Transmits a single UDP datagram over multicast, via IPv4 AND IPv6 (meaning at most two packets
    /// are sent overall).
    pub async fn tx_message(
        &self,
        message: Message,
    ) -> anyhow::Result<()>
    {
        // send IPv4 beacon if configured
        // let mut endpoint = self.endpoint.lock().await;
        if !self.endpoint.config().m_addr4.is_empty()
        {
            if self.endpoint.tx(PacketType::Ipv4, &message.to_string().as_bytes()).await?
            {
                info!("tx4-> {}", &message);
            }
        }

        // send IPv6 beacon if configured
        if !self.endpoint.config().m_addr6.is_empty()
        {
            if self.endpoint.tx(PacketType::Ipv6, &message.to_string().as_bytes()).await?
            {
                info!("tx6-> {}", &message);
            }
        }

        Ok(())
    }
}
