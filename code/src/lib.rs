mod protocol;

use protocol::{
    discovery::AdvertisementMessage,
    download::DownloadMessage,
    n_digit_number::{
        FourDigitNumber,
        ThreeDigitNumber,
        TwoDigitNumber,
    },
    search::SearchMessage,
};
use std::{
    fmt::{self, Formatter},
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
