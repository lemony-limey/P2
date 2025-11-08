use std::str::from_utf8;
use anyhow::anyhow;
use code::{Header, Identifier, Message, Payload, SerialNumber, Timestamp};
use bytes::{Bytes};
use nom::{
    AsBytes,
    Err,
    IResult,
    Parser,
    bytes::complete::take,
    character::char,
    error::ErrorKind,
};
// Err(Err::Failure(nom::error::Error { input, code: ErrorKind::Fail }))

const PROTOCOL_DELIMITER: char = ':';

pub fn parse_message(input: Bytes) -> anyhow::Result<Message>
{
    let result = parse_general_message(str::from_utf8(&input)?);

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
    let (remaining_input, _parsed_delimiter) = char(PROTOCOL_DELIMITER).parse(input)?;

    // Parse the header
    let (remaining_input, header) = parse_header(remaining_input)?;

    // Parse the ":" character in the middle
    let (remaining_input, _parsed_delimiter) = char(PROTOCOL_DELIMITER).parse(input)?;

    // Parse the payload
    let (remaining_input, payload) = parse_payload(remaining_input)?;

    // Parse the final ":" character
    let (remaining_input, _parsed_delimiter) = char(PROTOCOL_DELIMITER).parse(input)?;

    Ok((
        remaining_input,
        Message { header, payload },
    ))
}

fn parse_header(input: &str) -> IResult<&str, Header>
{
    // Parse the identifier


    // Parse the delimiter
    let (remaining_input, _parsed_delimiter) = char(PROTOCOL_DELIMITER).parse(input)?;

    // Parse the serial number


    // Parse the delimiter
    let (remaining_input, _parsed_delimiter) = char(PROTOCOL_DELIMITER).parse(input)?;

    // Parse the timestamp


    todo!();
    Err(Err::Failure(nom::error::Error { input, code: ErrorKind::Fail }))
}

fn parse_identifier(input: &str) -> IResult<&str, Identifier>
{
    todo!()
}

fn parse_serial_number(input: &str) -> IResult<&str, SerialNumber>
{
    todo!()
}

fn parse_timestamp(input: &str) -> IResult<&str, Timestamp>
{
    let (remaining_input, year) = take(4usize).parse(input)?;

    todo!()
}

fn parse_payload(input: &str) -> IResult<&str, Payload>
{

    todo!()
}
