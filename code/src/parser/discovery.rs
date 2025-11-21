use std::str::FromStr;
use std::net::{Ipv6Addr, SocketAddrV6};
use crate::{
    constants::PROTOCOL_DELIMITER_STR,
    parser::message::parse_delimiter,
    protocol::{
        discovery::AdvertisementMessage,
        services::{DownloadAvailability, SearchAvailability, Services},
    },
};
use nom::{
    bytes::complete::take_until1,
    character::complete::{char, digit1},
    combinator::map_res,
    error::ErrorKind,
    IResult,
    Parser,
};
use std::num::NonZeroU16;
use crate::constants::{CLOSING_SQUARE_BRACKET_CHAR, OPENING_SQUARE_BRACKET_CHAR};

/// Parser for the Requirement 1 messages.
pub(crate) fn parse_advertisement_message(input: &str) -> IResult<&str, AdvertisementMessage>
{
    // Get the IPv6 link local address
    let (remaining_input, (ipv6_addr, server_port)) = parse_ipv6_addr_and_port(input)?;
    let (remaining_input, _delimiter) = parse_delimiter(remaining_input)?;

    // Parse the services
    let (remaining_input, services) = parse_services(remaining_input)?;

    Ok((
        remaining_input,
        AdvertisementMessage::new(
            ipv6_addr,
            server_port,
            services,
        )
    ))
}

pub(crate) fn parse_ipv6_addr_and_port(input: &str) -> IResult<&str, (SocketAddrV6, NonZeroU16)>
{
    // Parse the opening bracket
    let (remaining_input, _opening_bracket) = char(OPENING_SQUARE_BRACKET_CHAR).parse(input)?;

    // Parse the IP address in the middle up to the closing bracket (the closing bracket itself
    // is not consumed yet).
    let (remaining_input, ipv6_addr) = map_res(
        take_until1("]"),
        str::parse::<Ipv6Addr>,
    ).parse(remaining_input)?;

    // Parse the closing bracket and following delimiter
    let (remaining_input, _opening_bracket) = char(CLOSING_SQUARE_BRACKET_CHAR).parse(remaining_input)?;
    let (remaining_input, _delimiter) = parse_delimiter(remaining_input)?;

    // Get the server port number and parse the delimiter
    let (remaining_input, server_port) = map_res(
        digit1,
        str::parse::<NonZeroU16>
    ).parse(remaining_input)?;

    // Create a SocketAddrV6 from the IP and port number
    let Ok(socket_addr_v6) = SocketAddrV6::from_str(
        &format!("[{}]:{}", ipv6_addr, server_port)
    ) else {
        return Err(nom::Err::Failure(nom::error::Error { input, code: ErrorKind::Fail }));
    };

    Ok((
        remaining_input,
        (socket_addr_v6, server_port),
    ))
}

pub(crate) fn parse_services(input: &str) -> IResult<&str, Services>
{
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
            services.search = SearchAvailability(service_1_status);
            services.download = DownloadAvailability(service_2_status);
        },
        ("download", "search") => {
            services.search = SearchAvailability(service_2_status);
            services.download = DownloadAvailability(service_1_status);
        },
        _ => return Err(nom::Err::Failure(nom::error::Error { input, code: ErrorKind::Fail })),
    }

    Ok((
        remaining_input,
        services
    ))
}
