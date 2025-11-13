use anyhow::anyhow;
use bytes::Bytes;
use crate::{
    constants::{
        ADVERTISEMENT,
        DOWNLOAD,
        FOUR_DIGITS,
        IDENTIFIER_DELIMITER_CHAR,
        IDENTIFIER_DELIMITER_STR,
        PROTOCOL_DELIMITER_CHAR,
        PROTOCOL_DELIMITER_STR,
        SEARCH,
        THREE_DIGITS,
        TIMESTAMP_DELIMITER_1,
        TIMESTAMP_DELIMITER_2,
        TWO_DIGITS
    },
    parser::{
        discovery::parse_advertisement_message,
        download::parse_download_message,
        search::parse_search_message,
    },
    protocol::{
        message::{Header, Identifier, Message, Payload, ResponseID, Timestamp},
        serial_number::SerialNumber,
    }
};
use nom::{
    bytes::complete::{take, take_until1},
    character::complete::{char, digit1},
    combinator::map_res,
    error::ErrorKind,
    Err,
    Finish,
    IResult,
    Parser,
};

// Methods
/// This is the public method that will be called from the other crates.
pub fn parse_message(input: &Bytes) -> anyhow::Result<Message<'_>>
{
    let result = parse_message_inner(str::from_utf8(&input)?)
        .finish();

    if let Ok((_remaining_input, message)) = result
    {
        Ok(message)
    }
    else
    {
        Err(anyhow!(Err::Failure(nom::error::Error { input: input.clone(), code: ErrorKind::Fail })))
    }
}

fn parse_message_inner(input: &str) -> IResult<&str, Message<'_>>
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
        Message::new(header, payload),
    ))
}

pub(crate) fn parse_header(input: &str) -> IResult<&str, Header<'_>>
{
    // Parse the identifier and serial number
    let (remaining_input, response_id) = parse_response_id(input)?;

    // Parse the delimiter
    let (remaining_input, _delimiter) = parse_delimiter(remaining_input)?;

    // Parse the timestamp
    let (remaining_input, timestamp) = parse_timestamp(remaining_input)?;

    Ok((
        remaining_input,
        Header::new(
            response_id.identifier,
            response_id.serial_number,
            timestamp
        )
    ))
}

pub(crate) fn parse_response_id(input: &str) -> IResult<&str, ResponseID<'_>>
{
    let (remaining_input, identifier) = parse_identifier(input)?;
    let (remaining_input, _delimiter) = parse_delimiter(remaining_input)?;
    let (remaining_input, serial_number) = parse_serial_number(remaining_input)?;

    Ok((
        remaining_input,
        ResponseID::new(identifier, serial_number)
    ))
}

pub(crate) fn parse_identifier(input: &str) -> IResult<&str, Identifier<'_>>
{
    // Get the identifier, then consume the delimiter "@".
    let (remaining_input, uid) = take_until1(IDENTIFIER_DELIMITER_STR).parse(input)?;
    let (remaining_input, _delimiter) = char(IDENTIFIER_DELIMITER_CHAR).parse(remaining_input)?;

    // Get the FQDN, but don't consume the following delimiter. That is out of the scope of
    // this combinator, and will be handled by the caller (most likely parse_header()).
    let (remaining_input, fqdn) = take_until1(PROTOCOL_DELIMITER_STR).parse(remaining_input)?;

    Ok((
        remaining_input,
        Identifier::new(uid, fqdn)
    ))
}

pub(crate) fn parse_serial_number(input: &str) -> IResult<&str, SerialNumber>
{
    let (remaining_input, serial_number) = map_res(digit1, str::parse).parse(input)?;

    Ok((
        remaining_input,
        SerialNumber(serial_number)
    ))
}

pub(crate) fn parse_timestamp(input: &str) -> IResult<&str, Timestamp>
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
    let Ok(timestamp) = Timestamp::new(
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

pub(crate) fn parse_delimiter(input: &str) -> IResult<&str, char>
{
    char(PROTOCOL_DELIMITER_CHAR).parse(input)
}


/// Parses a payload from the input string.
fn parse_payload(input: &str) -> IResult<&str, Payload<'_>>
{
    // Get the type of the payload: "discovery", "download" or "search", then consume a delimiter
    let (remaining_input, payload_type) = take_until1(PROTOCOL_DELIMITER_STR).parse(input)?;
    let (remaining_input, _delimiter) = parse_delimiter(remaining_input)?;

    match payload_type.to_lowercase().as_str()
    {
        ADVERTISEMENT => {
            let (remaining_input, advertisement_message) = parse_advertisement_message(remaining_input)?;
            Ok((remaining_input, Payload::Advertisement(advertisement_message)))
        },
        DOWNLOAD => {
            let (remaining_input, download_message) = parse_download_message(remaining_input)?;
            Ok((remaining_input, Payload::Download(download_message)))
        },
        SEARCH => {
            let (remaining_input, search_message) = parse_search_message(remaining_input)?;
            Ok((remaining_input, Payload::Search(search_message)))
        }
        _ => Err(Err::Failure(nom::error::Error { input, code: ErrorKind::Fail })),
    }
}
