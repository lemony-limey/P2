pub mod constants;

use anyhow::anyhow;
use bytes::Bytes;
use code::{
    Header,
    Identifier,
    Message,
    Payload,
    protocol::{
        discovery::AdvertisementMessage,
        download::DownloadMessage,
        search::SearchMessage,
    },
    SerialNumber,
    Timestamp,
};
use crate::constants::{
    DISCOVERY,
    DOWNLOAD,
    FOUR_DIGITS,
    IDENTIFIER_DELIMITER_CHAR,
    IDENTIFIER_DELIMITER_STR,
    NUMBER_OF_SERVICES,
    PROTOCOL_DELIMITER_CHAR,
    PROTOCOL_DELIMITER_STR,
    SEARCH,
    THREE_DIGITS,
    TIMESTAMP_DELIMITER_1,
    TIMESTAMP_DELIMITER_2,
    TWO_DIGITS
};
use nom::{
    bytes::complete::{
        take,
        take_until1,
    },
    character::complete::{
        char,
        digit1,
    },
    combinator::map_res,
    error::ErrorKind,
    Err,
    Finish,
    IResult,
    Parser,
};
use std::num::NonZeroU16;
use nom::branch::alt;
use nom::bytes::complete::tag;
use code::protocol::discovery::Services;

// Methods
pub fn parse_message(input: Bytes) -> anyhow::Result<Message>
{
    let result = parse_general_message(str::from_utf8(&input)?)
        .finish();

    if let Ok((_remaining_input, message)) = result
    {
        Ok(message)
    }
    else
    {
        Err(anyhow!(Err::Failure(nom::error::Error { input, code: ErrorKind::Fail })))
    }
}

fn parse_general_message(input: &str) -> IResult<&str, Message>
{
    // Parse the first ":" character
    let (remaining_input, _delimiter) = parse_delimiter(input)?;

    // Parse the header
    let (remaining_input, header) = parse_header(remaining_input)?;

    // Parse the ":" character in the middle
    let (remaining_input, _delimiter) = parse_delimiter(remaining_input)?;

    // Parse the payload
    let (remaining_input, payload) = parse_payload(remaining_input)?;

    // Parse the final ":" character
    let (remaining_input, _delimiter) = parse_delimiter(remaining_input)?;

    Ok((
        remaining_input,
        Message { header, payload },
    ))
}

fn parse_header(input: &str) -> IResult<&str, Header>
{
    // Parse the identifier
    let (remaining_input, identifier) = parse_identifier(input)?;

    // Parse the delimiter
    let (remaining_input, _delimiter) = parse_delimiter(remaining_input)?;

    // Parse the serial number
    let (remaining_input, serial_number) = parse_serial_number(remaining_input)?;

    // Parse the delimiter
    let (remaining_input, _delimiter) = parse_delimiter(remaining_input)?;

    // Parse the timestamp
    let (remaining_input, timestamp) = parse_timestamp(remaining_input)?;

    Ok((
        remaining_input,
        Header { identifier, serial_number, timestamp }
    ))
}

fn parse_identifier(input: &str) -> IResult<&str, Identifier>
{
    // Get the identifier, then consume the delimiter "@".
    let (remaining_input, uid) = take_until1(IDENTIFIER_DELIMITER_STR).parse(input)?;
    let (remaining_input, _delimiter) = char(IDENTIFIER_DELIMITER_CHAR).parse(remaining_input)?;

    // Get the FQDN, but don't consume the following delimiter. That is out of the scope of
    // this combinator, and will be handled by the caller (most likely parse_header()).
    let (remaining_input, fqdn) = take_until1(PROTOCOL_DELIMITER_STR).parse(remaining_input)?;

    Ok((
        remaining_input,
        Identifier
        {
            uid: uid.to_string(),
            fqdn: fqdn.to_string(),
        }
    ))
}

fn parse_serial_number(input: &str) -> IResult<&str, SerialNumber>
{
    let (remaining_input, serial_number) = map_res(digit1, str::parse).parse(input)?;

    Ok((
        remaining_input,
        SerialNumber(serial_number)
    ))
}

fn parse_timestamp(input: &str) -> IResult<&str, Timestamp>
{
    // Year, Month, Day
    let (remaining_input, year) = map_res(take(FOUR_DIGITS), str::parse::<u16>).parse(input)?;
    let (remaining_input, month) = map_res(take(TWO_DIGITS), str::parse::<u8>).parse(remaining_input)?;
    let (remaining_input, day) = map_res(take(TWO_DIGITS), str::parse::<u8>).parse(remaining_input)?;

    // First delimiter, "-"
    let (remaining_input, _delimiter) = char(TIMESTAMP_DELIMITER_1).parse(remaining_input)?;

    // Hours, Minutes, Seconds
    let (remaining_input, hours) = map_res(take(TWO_DIGITS), str::parse::<u8>).parse(remaining_input)?;
    let (remaining_input, minutes) = map_res(take(TWO_DIGITS), str::parse::<u8>).parse(remaining_input)?;
    let (remaining_input, seconds) = map_res(take(TWO_DIGITS), str::parse::<u8>).parse(remaining_input)?;

    // Second delimiter, "."
    let (remaining_input, _delimiter) = char(TIMESTAMP_DELIMITER_2).parse(remaining_input)?;

    // Milliseconds
    let (remaining_input, milliseconds) = map_res(take(THREE_DIGITS), str::parse::<u16>).parse(remaining_input)?;

    // Attempt to create a Timestamp object with the provided values.
    let Ok(timestamp) = Timestamp::try_new(
        year,
        month,
        day,
        hours,
        minutes,
        seconds,
        milliseconds,
    ) else {
        return Err(Err::Failure(nom::error::Error { input, code: ErrorKind::Fail }));
    };

    Ok((
        remaining_input,
        timestamp,
    ))
}

fn parse_delimiter(input: &str) -> IResult<&str, char>
{
    char(PROTOCOL_DELIMITER_CHAR).parse(input)
}

fn parse_payload(input: &str) -> IResult<&str, Payload>
{
    // Get the type of the payload: "discovery", "download" or "search", then consume a delimiter
    let (remaining_input, payload_type) = take_until1(PROTOCOL_DELIMITER_STR).parse(input)?;
    let (remaining_input, _delimiter) = parse_delimiter(remaining_input)?;

    let result: (&str, Payload);
    match payload_type.to_lowercase().as_str()
    {
        DISCOVERY => {
            let (remaining_input, advertisement_message) = parse_discovery_message(remaining_input)?;
            result = (remaining_input, Payload::Discovery(advertisement_message))
        },
        DOWNLOAD => {
            let (remaining_input, download_message) = parse_download_message(remaining_input)?;
            result = (remaining_input, Payload::Download(download_message))
        },
        SEARCH => {
            let (remaining_input, search_message) = parse_search_message(remaining_input)?;
            result = (remaining_input, Payload::Search(search_message))
        }
        _ => return Err(Err::Failure(nom::error::Error { input, code: ErrorKind::Fail })),
    }

    Ok(result)
}

fn parse_discovery_message(input: &str) -> IResult<&str, AdvertisementMessage>
{
    // Get the server port number
    let (remaining_input, server_port) = map_res(digit1, str::parse::<NonZeroU16>).parse(input)?;
    let (remaining_input, _delimiter) = parse_delimiter(remaining_input)?;

    // Parse the services
    let (remaining_input, services) = parse_services(remaining_input)?;

    Ok((
        remaining_input,
        AdvertisementMessage { server_port, services }
    ))
}

fn parse_services(input: &str) -> IResult<&str, Services>
{
    // Create arrays to
    // let mut service_names: Vec<&str> = Vec::with_capacity(NUMBER_OF_SERVICES);
    // let mut service_statuses: [bool; NUMBER_OF_SERVICES] = [false; NUMBER_OF_SERVICES];

    // Get the name of the first service and its status.
    let (remaining_input, service_1_name) = take_until1("=").parse(input)?;
    let (remaining_input, _delimiter) = char('=').parse(remaining_input)?;

    // Get the status of the first service.
    let (remaining_input, service_1_status) = map_res(
        take_until1(","),
        str::parse::<bool>
    ).parse(remaining_input)?;
    let (remaining_input, _delimiter) = char(',').parse(remaining_input)?;

    // Get the name of the second service and its status.
    let (remaining_input, service_2_name) = take_until1("=").parse(remaining_input)?;
    let (remaining_input, _delimiter) = char('=').parse(remaining_input)?;
    let (remaining_input, service_2_status) = map_res(
        take_until1(PROTOCOL_DELIMITER_STR),
        str::parse::<bool>
    ).parse(remaining_input)?;

    let mut services = Services::default();

    match (service_1_name, service_2_name)
    {
        ("search", "download") => {
            services.search = service_1_status;
            services.download = service_2_status;
        },
        ("download", "search") => {
            services.search = service_2_status;
            services.download = service_1_status;
        },
        _ => return Err(Err::Failure(nom::error::Error { input, code: ErrorKind::Fail })),
    }

    Ok((
        remaining_input,
        services
    ))
}

fn parse_download_message(input: &str) -> IResult<&str, DownloadMessage>
{
    todo!()
}

fn parse_search_message(input: &str) -> IResult<&str, SearchMessage>
{
    todo!()
}
