pub mod discovery;
pub mod download;
pub mod identifier;
pub mod message;
pub mod search;

#[cfg(test)]
mod tests
{
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


    #[test]
    fn advertisement_1()
    {
        let timestamp = Timestamp::now().unwrap();
        let server_port_number = NonZeroU16::new(23756)
            .expect("invalid server port number");
        let message = Message::new(
            Header::new(
                Identifier::new(
                    UID("uid".to_string()),
                    FQDN("cs.st-andrews.ac.uk".to_string()),
                ),
                SerialNumber(
                    NonZeroU64::new(1).expect("failed to create serial number")
                ),
                timestamp.clone(),
            ),
            Payload::Advertisement(
                AdvertisementMessage::new(
                    server_port_number,
                    Services::new(
                        SearchAvailability(true),
                        DownloadAvailability(false)
                    ),
                )
            ),
        );

        assert_eq!(
            message.to_string(),
            format!(
                ":uid@cs.st-andrews.ac.uk:1:{}:advertisement:23756:search=true,download=false:",
                timestamp
            )
        )
    }
}
