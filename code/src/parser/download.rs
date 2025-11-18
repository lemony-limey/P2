use crate::{
    constants::{ERROR, PROTOCOL_DELIMITER_STR, REQUEST, RESULT},
    parser::message::{parse_delimiter, parse_response_id},
    protocol::download::DownloadMessage,
};
use nom::{
    bytes::complete::take_until1,
    error::ErrorKind,
    IResult,
    Parser,
};

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
    let (remaining_input, file_string) = take_until1(PROTOCOL_DELIMITER_STR).parse(input)?;

    Ok((
        remaining_input,
        DownloadMessage::Request { file_string: file_string.to_string() },
    ))
}

pub(crate) fn parse_download_result_message(input: &str) -> IResult<&str, DownloadMessage>
{
    let (remaining_input, _response_id) = parse_response_id(input)?;
    let (remaining_input, _delimiter) = parse_delimiter(remaining_input)?;

    // Get the file string, but don't consume the delimiter after it.
    let (remaining_input, _file_string) = take_until1(PROTOCOL_DELIMITER_STR).parse(remaining_input)?;

    todo!()
    // Ok((
    //     remaining_input,
    //
    //     ))
}
