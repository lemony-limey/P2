use std::fmt;
use std::fmt::Formatter;
use std::marker::PhantomData;

#[derive(Copy, Clone, Debug)]
pub struct UID<'a>(pub &'a str);
#[derive(Copy, Clone, Debug)]
pub struct FQDN<'a>(pub &'a str);

#[derive(Copy, Clone, Debug)]
pub struct Identifier<'a>
{
    uid:  UID<'a>,
    fqdn: FQDN<'a>,
}

impl<'a> Identifier<'a>
{
    pub fn new(uid: UID<'a>, fqdn: FQDN<'a>) -> Self
    {
        Self
        {
            uid,
            fqdn,
        }
    }

    pub fn uid(&self) -> &'a str
    {
        self.uid.0
    }

    pub fn fqdn(&self) -> &'a str
    {
        self.fqdn.0
    }
}

impl<'a> fmt::Display for Identifier<'a>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        write!(f, "{}@{}", self.uid(), self.fqdn())
    }
}
