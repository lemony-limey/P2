pub mod constants;
pub mod n_digit_number;
pub mod parser;
pub mod protocol;

#[cfg(test)]
mod tests
{
    use crate::protocol::services::DownloadAvailability;
    use crate::protocol::services::SearchAvailability;
    use crate::protocol::services::Services;
    use crate::protocol::discovery::AdvertisementMessage;
    use crate::protocol::message::{Payload, SerialNumber, Timestamp};
    use crate::protocol::message::Identifier;
    use crate::protocol::message::Header;
    use crate::protocol::message::Message;
    use std::num::{NonZeroU16, NonZeroU64};

    #[test]
    fn advertisement_1()
    {
        let server_port_number = NonZeroU16::new(23756)
            .expect("invalid serverport number");
        let message = Message::new(
            Header::new(
                Identifier::new(
                    "uid",
                    "cs.st-andrews.ac.uk",
                ),
                SerialNumber(
                    NonZeroU64::new(1).expect("failed to create serial number")
                ),
                Timestamp::current_time().unwrap(),
            ),
            Payload::Advertisement(AdvertisementMessage::new(
                server_port_number,
                Services::new(SearchAvailability(true), DownloadAvailability(false)),
            )),
        );

        assert_eq!(
            message.to_string(),
            "::advertisement:23756:search=true,download=false:"
        )
    }
}
