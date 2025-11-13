use chrono::{Datelike, SubsecRound, Timelike};
use crate::protocol::{
    discovery::AdvertisementMessage,
    download::DownloadMessage,
    search::SearchMessage,
};
use std::{
    fmt::{self, Formatter},
    marker::PhantomData,
    num::{NonZeroU64, ParseIntError},
    str::FromStr,
};
use crate::constants::ONE_MILLION;
use crate::n_digit_number::{
    FourDigitNumber,
    ThreeDigitNumber,
    TwoDigitNumber,
};

/// General Message Format
#[derive(Clone, Debug)]
pub struct Message<'a>
{
    pub header:  Header<'a>,
    pub payload: Payload<'a>,
    // Force the new() constructor to be used.
    _boo: PhantomData<usize>,
}

impl<'a> Message<'a>
{
    pub fn new(header: Header<'a>, payload: Payload<'a>) -> Self
    {
        Self
        {
            header,
            payload,
            _boo: Default::default(),
        }
    }
}

impl fmt::Display for Message<'_>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        write!(f, ":{}:{}:", self.header, self.payload)
    }
}


#[derive(Clone, Debug)]
pub struct Header<'a>
{
    pub identifier:    Identifier<'a>,
    pub serial_number: SerialNumber,
    pub timestamp:     Timestamp,
    // Force the new() constructor to be used.
    _boo: PhantomData<usize>,
}

impl<'a> Header<'a>
{
    pub fn new(
        identifier:    Identifier<'a>,
        serial_number: SerialNumber,
        timestamp:     Timestamp
    ) -> Self
    {
        Self
        {
            identifier,
            serial_number,
            timestamp,
            _boo: Default::default(),
        }
    }
}

impl fmt::Display for Header<'_>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        write!(f, "{}:{}:{}", self.identifier, self.serial_number, self.timestamp)
    }
}


#[derive(Clone, Debug)]
pub struct Identifier<'a>
{
    pub uid:  &'a str,
    pub fqdn: &'a str,
    // Force the new() constructor to be used.
    _boo: PhantomData<usize>,
}

impl<'a> Identifier<'a>
{
    pub fn new(uid: &'a str, fqdn: &'a str) -> Self
    {
        Self
        {
            uid,
            fqdn,
            _boo: Default::default(),
        }
    }
}

impl<'a> fmt::Display for Identifier<'a>
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
    pub fn new(
        year:         u16,
        month:        u8,
        day:          u8,
        hours:        u8,
        minutes:      u8,
        seconds:      u8,
        milliseconds: u16,
    ) -> anyhow::Result<Self>
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

    pub fn current_time() -> anyhow::Result<Self>
    {
        let current_time = chrono::Utc::now();

        Self::new(
            current_time.year() as u16,
            current_time.month() as u8,
            current_time.day() as u8,
            current_time.hour() as u8,
            current_time.minute() as u8,
            current_time.second() as u8,
            (current_time.round_subsecs(3).nanosecond() / ONE_MILLION) as u16
        )
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
pub enum Payload<'a>
{
    Advertisement(AdvertisementMessage),
    Search(SearchMessage<'a>),
    Download(DownloadMessage<'a>),
}

impl fmt::Display for Payload<'_>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        match self
        {
            Payload::Advertisement(message) => {
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
pub struct ResponseID<'a>
{
    pub identifier:    Identifier<'a>,
    pub serial_number: SerialNumber,
    // Force the new() constructor to be used.
    _boo: PhantomData<usize>,
}

impl<'a> ResponseID<'a>
{
    pub fn new(identifier: Identifier<'a>, serial_number: SerialNumber) -> Self
    {
        Self
        {
            identifier,
            serial_number,
            _boo: Default::default(),
        }
    }
}

impl fmt::Display for ResponseID<'_>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        write!(f, "{}:{}", self.identifier, self.serial_number)
    }
}
