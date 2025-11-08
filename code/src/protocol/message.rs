use crate::protocol::{
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
    num::ParseIntError,
    str::FromStr,
};
use std::marker::PhantomData;
use std::num::NonZeroU64;

/// General Message Format
#[derive(Clone, Debug)]
pub struct Message
{
    pub header:  Header,
    pub payload: Payload,
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
    pub identifier:    Identifier,
    pub serial_number: SerialNumber,
    pub timestamp:     Timestamp,
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
    pub uid:  String,
    pub fqdn: String,
}

impl fmt::Display for Identifier
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        write!(f, "{}@{}", self.uid, self.fqdn)
    }
}


#[derive(Clone, Debug)]
pub struct SerialNumber(pub NonZeroU64);

impl fmt::Display for SerialNumber
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        write!(f, "{}", self.0)
    }
}

impl FromStr for SerialNumber
{
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err>
    {
        let result = str::parse(s)?;
        Ok(Self(result))
    }
}


#[derive(Clone, Debug)]
pub struct Timestamp
{
    pub year:         FourDigitNumber,
    pub month:        TwoDigitNumber,
    pub day:          TwoDigitNumber,
    pub hours:        TwoDigitNumber,
    pub minutes:      TwoDigitNumber,
    pub seconds:      TwoDigitNumber,
    pub milliseconds: ThreeDigitNumber,
    // PhantomData doesn't actually use up any space, it just acts as a private
    // member in the struct. Thus, since not every struct member is public, the default
    // constructor cannot be freely used, and the new() function must be used instead.
    _boo: PhantomData<usize>,
}

impl Timestamp
{
    pub fn try_new(
        year: u16,
        month: u8,
        day: u8,
        hours: u8,
        minutes: u8,
        seconds: u8,
        milliseconds: u16,
    ) -> Result<Self, String>
    {
        let year = FourDigitNumber::new(year)?;
        let month = TwoDigitNumber::new(month)?;
        let day = TwoDigitNumber::new(day)?;
        let hours = TwoDigitNumber::new(hours)?;
        let minutes = TwoDigitNumber::new(minutes)?;
        let seconds = TwoDigitNumber::new(seconds)?;
        let milliseconds = ThreeDigitNumber::new(milliseconds)?;

        Ok(Self { year, month, day, hours, minutes, seconds, milliseconds, _boo: Default::default() })
    }
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
    pub identifier:    Identifier,
    pub serial_number: SerialNumber,
}

impl fmt::Display for ResponseID
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        write!(f, "{}:{}", self.identifier, self.serial_number)
    }
}
