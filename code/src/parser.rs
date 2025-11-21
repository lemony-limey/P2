pub mod discovery;
pub mod download;
pub mod identifier;
pub mod message;
pub mod search;

#[cfg(test)]
mod tests
{
    use bytes::BytesMut;
    use std::net::Ipv6Addr;
    use crate::protocol::{
        discovery::AdvertisementMessage,
        identifier::{FQDN, Identifier, UID},
        message::{
            Header,
            Message,
            Payload,
            Timestamp,
        },
        serial_number::SerialNumber,
        services::{
            DownloadAvailability,
            SearchAvailability,
            Services,
        },
    };
    use std::num::{NonZeroU16, NonZeroU64};
    use crate::logical_file_path::LogicalFilePath;
    use crate::parser::message::parse_message;
    use crate::protocol::download::DownloadMessage;

    fn generate_header(timestamp: &Timestamp) -> Header
    {
        Header::new(
            Identifier::new(
                UID("uid".to_string()),
                FQDN("cs.st-andrews.ac.uk".to_string()),
            ),
            SerialNumber(
                NonZeroU64::new(1).expect("failed to create serial number")
            ),
            timestamp.clone(),
        )
    }

    #[test]
    fn advertisement_1()
    {
        let timestamp = Timestamp::now().unwrap();
        let server_port_number = NonZeroU16::new(23756)
            .expect("invalid server port number");
        let ipv6_addr = "::1".parse::<Ipv6Addr>().unwrap();
        let message = Message::new(
            generate_header(&timestamp),
            Payload::Advertisement(
                AdvertisementMessage::new(
                    ipv6_addr,
                    server_port_number,
                    Services::new(
                        SearchAvailability(true),
                        DownloadAvailability(false)
                    ),
                )
            ),
        );

        // Test conversion from message object to string
        let message_string = format!(
            ":uid@cs.st-andrews.ac.uk:1:{}:advertisement:(::1):23756:search=true,download=false:",
            timestamp
        );
        assert_eq!(
            message.to_string(),
            message_string,
        );

        let message_bytes = BytesMut::from(message_string.as_str());
        let parsed_message = parse_message(&message_bytes).expect("parsing failed");

        // Test conversion from string to message object
        assert_eq!(
            parsed_message,
            message,
        );
    }

    #[test]
    fn test_leaving()
    {
        let timestamp = Timestamp::now().unwrap();
        let message = Message::new(
            generate_header(&timestamp),
            Payload::Leaving,
        );

        // Test conversion from message object to string
        let message_string = format!(
            ":uid@cs.st-andrews.ac.uk:1:{}:leaving:",
            timestamp
        );
        assert_eq!(
            message.to_string(),
            message_string,
        );

        let message_bytes = BytesMut::from(message_string.as_str());
        let parsed_message = parse_message(&message_bytes).expect("parsing failed");

        // Test conversion from string to message object
        assert_eq!(
            parsed_message,
            message,
        );
    }

    #[test]
    fn test_download_result()
    {
        let timestamp = Timestamp::now().unwrap();
        let message = Message::new(
            generate_header(&timestamp),
            Payload::Download(
                DownloadMessage::Request {
                    identifier: Identifier::new(
                        UID("uid2".to_string()),
                        FQDN("cs.st-andrews.ac.uk".to_string())
                    ),
                    logical_file_path: LogicalFilePath::new("/dir1/text1-1.txt".to_string()),
                },
            ),
        );

        // Test conversion from message object to string
        let message_string = format!(
            ":uid@cs.st-andrews.ac.uk:1:{}:download:request:uid2@cs.st-andrews.ac.uk:/dir1/text1-1.txt:",
            timestamp
        );
        assert_eq!(
            message.to_string(),
            message_string,
        );

        let message_bytes = BytesMut::from(message_string.as_str());
        let parsed_message = parse_message(&message_bytes).expect("parsing failed");

        // Test conversion from string to message object
        assert_eq!(
            parsed_message,
            message,
        );
    }
}
