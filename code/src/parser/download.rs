use crate::{
    constants::{PROTOCOL_DELIMITER_STR, REQUEST, RESULT},
    parser::message::{parse_delimiter, parse_response_id},
    protocol::download::DownloadMessage,
};
use nom::{
    bytes::complete::take_until1,
    error::ErrorKind,
    IResult,
    Parser,
};
use nom::character::complete::digit1;
use nom::combinator::map_res;
use crate::logical_file_path::LogicalFilePath;
use crate::parser::identifier::parse_identifier;

/// Parser for the Requirement 3 messages.
pub(crate) fn parse_download_message(input: &str) -> IResult<&str, DownloadMessage>
{
    let (remaining_input, message_type) = take_until1(PROTOCOL_DELIMITER_STR).parse(input)?;
    let (remaining_input, _delimiter) = parse_delimiter(remaining_input)?;

    match message_type.to_lowercase().as_str()
    {
        REQUEST => {
            Ok(parse_download_request_message(remaining_input)?)
        },
        RESULT => {
            Ok(parse_download_result_message(remaining_input)?)
        },
        _ => Err(nom::Err::Failure(nom::error::Error { input, code: ErrorKind::Fail })),
    }
}

pub(crate) fn parse_download_request_message(input: &str) -> IResult<&str, DownloadMessage>
{
    // Parse the identifier and the delimiter that follows it
    let (remaining_input, identifier) = parse_identifier(input)?;
    let (remaining_input, _delimiter) = parse_delimiter(remaining_input)?;

    // Parse the logical filename
    let (remaining_input, logical_file_path) = take_until1(PROTOCOL_DELIMITER_STR).parse(remaining_input)?;

    Ok((
        remaining_input,
        DownloadMessage::Request {
            identifier,
            logical_file_path: LogicalFilePath::new(logical_file_path),
        },
    ))
}

pub(crate) fn parse_download_result_message(input: &str) -> IResult<&str, DownloadMessage>
{
    // Get the file string and consume the following delimiter.
    let (remaining_input, file_path) = take_until1(PROTOCOL_DELIMITER_STR).parse(input)?;
    let (remaining_input, _delimiter) = parse_delimiter(remaining_input)?;

    // Get the size of the file, but don't consume the following delimiter
    let (remaining_input, file_size) = map_res(digit1, str::parse::<u64>).parse(remaining_input)?;

    Ok((
        remaining_input,
        DownloadMessage::Result {
            logical_file_path: LogicalFilePath::new(file_path),
            file_size,
        },
    ))
}
