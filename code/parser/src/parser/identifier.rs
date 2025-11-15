use crate::{
    constants::{IDENTIFIER_DELIMITER_CHAR, IDENTIFIER_DELIMITER_STR, PROTOCOL_DELIMITER_STR},
    protocol::identifier::{Identifier, FQDN, UID},
};
use nom::{
    Parser,
    bytes::complete::take_until1,
    character::complete::char,
    IResult
};

pub(crate) fn parse_identifier(input: &str) -> IResult<&str, Identifier>
{
    // Get the identifier, then consume the delimiter "@".
    let (remaining_input, uid) = take_until1(IDENTIFIER_DELIMITER_STR).parse(input)?;
    let (remaining_input, _delimiter) = char(IDENTIFIER_DELIMITER_CHAR).parse(remaining_input)?;

    // Get the FQDN, but don't consume the following delimiter. That is out of the scope of
    // this combinator, and will be handled by the caller (most likely parse_header()).
    let (remaining_input, fqdn) = take_until1(PROTOCOL_DELIMITER_STR).parse(remaining_input)?;

    Ok((
        remaining_input,
        Identifier::new(
            UID(uid.to_string()),
            FQDN(fqdn.to_string()),
        )
    ))
}
