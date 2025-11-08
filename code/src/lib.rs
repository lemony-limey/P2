mod protocol;

use protocol::n_digit_number::{
    FourDigitNumber,
    ThreeDigitNumber,
    TwoDigitNumber,
};
use std::{
    fmt::{self, Formatter},
    num::NonZeroU16,
};

// General Message Format
#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
pub struct Identifier
{
    uid:  u32,
    fqdn: String,
}

impl fmt::Display for Identifier
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        write!(f, "{}@{}", self.uid, self.fqdn)
    }
}

#[derive(Clone, Debug)]
pub struct SerialNumber(u64);

impl fmt::Display for SerialNumber
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        write!(f, "{}", self.0)
    }
}


#[derive(Clone, Debug)]
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


#[derive(Clone, Debug)]
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
        match self
        {
            Payload::Discovery(message) => {
                write!(f, "{}", message)
            }
            Payload::Search(message) => {
                write!(f, "{}", message)
            }
            Payload::Download(message) => {
                write!(f, "{}", message)
            }
        }
    }
}


#[derive(Clone, Debug)]
pub struct ResponseID
{
    identifier:    Identifier,
    serial_number: SerialNumber,
}

impl fmt::Display for ResponseID
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        write!(f, "{}:{}", self.identifier, self.serial_number)
    }
}


#[derive(Clone, Debug)]
pub struct AdvertisementMessage
{
    server_port: NonZeroU16,
    services:    Services,
}

impl fmt::Display for AdvertisementMessage
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        write!(f, "advertisement:{}:{}", self.server_port, self.services)
    }
}


#[derive(Clone, Debug)]
pub struct Services
{
    search:   bool,
    download: bool,
}

impl fmt::Display for Services
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        write!(f, "search={},download={}", self.search, self.download)
    }
}


// R2 - Search
#[derive(Clone, Debug)]
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

impl fmt::Display for SearchMessage
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        match self
        {
            SearchMessage::SearchRequest(request) => {
                write!(f, "search-request:{}", request)
            }
            SearchMessage::SearchResult { response_id, file_string } => {
                write!(f, "search-result:{}:{}", response_id, file_string)
            }
            SearchMessage::SearchError { response_id } => {
                write!(f, "search-error:{}", response_id)
            }
        }
    }
}


// R3 - Download
#[derive(Clone, Debug)]
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

impl fmt::Display for DownloadMessage
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        match self
        {
            DownloadMessage::DownloadRequest { .. } => {
                todo!()
            }
            DownloadMessage::DownloadResult { .. } => {
                todo!()
            }
            DownloadMessage::DownloadError { .. } => {
                todo!()
            }
        }
    }
}
