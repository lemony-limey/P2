use std::fmt;
use std::fmt::Formatter;
use anyhow::Context;
use std::num::{NonZeroU64, ParseIntError};
use std::str::FromStr;

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


#[derive(Copy, Clone, Debug)]
pub struct SerialNumberGenerator
{
    next_serial_number: NonZeroU64,
}

impl SerialNumberGenerator
{
    pub fn new() -> anyhow::Result<Self>
    {
        let next_serial_number = NonZeroU64::new(1u64)
            .with_context(|| "SerialNumberGenerator::new(): this should never fail")?;
        Ok(Self { next_serial_number })
    }

    pub fn get_next_serial_number(&mut self) -> NonZeroU64
    {
        let serial_number = self.next_serial_number;
        let _ = self.next_serial_number.saturating_add(1u64);

        serial_number
    }
}

impl Default for SerialNumberGenerator
{
    fn default() -> Self
    {
        Self::new().expect("SerialNumberGenerator::default()")
    }
}
