use crate::{
    constants::{PROTOCOL_DELIMITER_STR, REQUEST, RESULT},
    parser::message::{parse_delimiter, parse_response_id},
    protocol::search::SearchMessage,
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

/// Parser for the Requirement 2 messages.
pub(crate) fn parse_search_message(input: &str) -> IResult<&str, SearchMessage>
{
    let (remaining_input, message_type) = take_until1(PROTOCOL_DELIMITER_STR).parse(input)?;
    let (remaining_input, _delimiter) = parse_delimiter(remaining_input)?;

    match message_type.to_lowercase().as_str()
    {
        REQUEST => {
            Ok(parse_search_request_message(remaining_input)?)
        },
        RESULT => {
            Ok(parse_search_result_message(remaining_input)?)
        },
        _ => Err(nom::Err::Failure(nom::error::Error { input, code: ErrorKind::Fail })),
    }
}

pub(crate) fn parse_search_request_message(input: &str) -> IResult<&str, SearchMessage>
{
    let (remaining_input, search_string) = take_until1(PROTOCOL_DELIMITER_STR).parse(input)?;

    Ok((
        remaining_input,
        SearchMessage::Request { search_string: search_string.to_string() },
    ))
}

pub(crate) fn parse_search_result_message(input: &str) -> IResult<&str, SearchMessage>
{
    // Get the Response ID and consume the following delimiter.
    let (remaining_input, response_id) = parse_response_id(input)?;
    let (remaining_input, _delimiter) = parse_delimiter(remaining_input)?;

    // Get the file string and consume the following delimiter.
    let (remaining_input, logical_file_path) = take_until1(PROTOCOL_DELIMITER_STR).parse(remaining_input)?;
    let (remaining_input, _delimiter) = parse_delimiter(remaining_input)?;

    // Get the size of the file, but don't consume the following delimiter
    let (remaining_input, file_size) = map_res(digit1, str::parse::<u64>).parse(remaining_input)?;

    Ok((
        remaining_input,
        SearchMessage::Result {
            response_id,
            logical_file_path: LogicalFilePath::new(logical_file_path),
            file_size,
        }
    ))
}
