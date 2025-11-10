use std::{
    fmt::{self, Formatter},
};

const TWO_DIGIT_NUMBER_MAX: u8 = 99;
const THREE_DIGIT_NUMBER_MAX: u16 = 999;
const FOUR_DIGIT_NUMBER_MAX: u16 = 9999;


#[derive(Copy, Clone, Debug)]
pub struct TwoDigitNumber { value: u8 }
#[derive(Copy, Clone, Debug)]
pub struct ThreeDigitNumber { value: u16 }
#[derive(Copy, Clone, Debug)]
pub struct FourDigitNumber { value: u16 }


impl TwoDigitNumber
{
    pub fn new(value: u8) -> Result<Self, String>
    {
        if value <= TWO_DIGIT_NUMBER_MAX
        {
            Ok(Self { value })
        }
        else
        {
            Err(format!("Cannot convert \"{value}\" into a TwoDigitNumber"))
        }
    }
}

impl fmt::Display for TwoDigitNumber
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        write!(f, "{:02}", self.value)
    }
}


impl ThreeDigitNumber
{
    pub fn new(value: u16) -> Result<Self, String>
    {
        if value <= THREE_DIGIT_NUMBER_MAX
        {
            Ok(Self { value })
        }
        else
        {
            Err(format!("Cannot convert \"{value}\" into a ThreeDigitNumber"))
        }
    }
}

impl fmt::Display for ThreeDigitNumber
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        write!(f, "{:03}", self.value)
    }
}


impl FourDigitNumber
{
    pub fn new(value: u16) -> Result<Self, String>
    {
        if value <= FOUR_DIGIT_NUMBER_MAX
        {
            Ok(Self { value })
        }
        else
        {
            Err(format!("Cannot convert \"{value}\" into a FourDigitNumber"))
        }
    }
}

impl fmt::Display for FourDigitNumber
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        write!(f, "{:04}", self.value)
    }
}
