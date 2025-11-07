mod n_digit_number;

use crate::n_digit_number::{
    TwoDigitNumber,
    ThreeDigitNumber,
    FourDigitNumber,
};
use std::{
    fmt::{self, Formatter},
    num::NonZeroU16,
};

// General Message Format
#[derive(Debug)]
pub struct Message
{
    header:  Header,
    payload: Payload,
}

impl fmt::Display for Message
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        write!(f, ":{}:{}:", self.header, self.payload)
    }
}

#[derive(Debug)]
pub struct Header
{
    identifier:    Identifier,
    serial_number: SerialNumber,
    timestamp:     Timestamp,
}

impl fmt::Display for Header
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        write!(f, "{}:{}:{}", self.identifier, self.serial_number, self.timestamp)
    }
}

#[derive(Debug)]
pub struct Identifier
{
    uid:  u32,
    fqdn: String,
}

impl fmt::Display for Identifier
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

#[derive(Debug)]
pub struct SerialNumber(u64);

impl fmt::Display for SerialNumber
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        write!(f, "{}", self.0)
    }
}


#[derive(Debug)]
pub struct Timestamp
{
    year:         FourDigitNumber,
    month:        TwoDigitNumber,
    day:          TwoDigitNumber,
    hours:        TwoDigitNumber,
    minutes:      TwoDigitNumber,
    seconds:      TwoDigitNumber,
    milliseconds: ThreeDigitNumber,
}

impl fmt::Display for Timestamp
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        write!(
            f,
            "{}{}{}-{}{}{}.{}",
            self.year,
            self.month,
            self.day,
            self.hours,
            self.minutes,
            self.seconds,
            self.milliseconds,
        )
    }
}


#[derive(Debug)]
pub enum Payload
{
    Discovery(AdvertisementMessage),
    Search(SearchMessage),
    Download(DownloadMessage),
}

impl fmt::Display for Payload
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        todo!()
    }
}


#[derive(Debug)]
pub struct ResponseID
{
    identifier:    Identifier,
    serial_number: SerialNumber,
}


#[derive(Debug)]
pub struct AdvertisementMessage
{
    server_port: NonZeroU16,
    services:    String,
}


// R2 - Search
#[derive(Debug)]
pub enum SearchMessage
{
    SearchRequest(String),
    SearchResult
    {
        response_id: ResponseID,
        file_string: String,
    },
    SearchError
    {
        response_id: ResponseID,
    },
}

#[derive(Debug)]
pub enum DownloadMessage
{
    DownloadRequest
    {

    },
    DownloadResult
    {

    },
    DownloadError
    {

    },
}
