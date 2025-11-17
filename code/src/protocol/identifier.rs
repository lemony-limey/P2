use std::fmt;
use std::fmt::Formatter;
use std::marker::PhantomData;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct UID(pub String);
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct FQDN(pub String);

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Identifier
{
    uid:  UID,
    fqdn: FQDN,
}

impl Identifier
{
    pub fn new(uid: UID, fqdn: FQDN) -> Self
    {
        Self
        {
            uid,
            fqdn,
        }
    }

    pub fn uid(&self) -> &String
    {
        &self.uid.0
    }

    pub fn fqdn(&self) -> &String
    {
        &self.fqdn.0
    }
}

impl<'a> fmt::Display for Identifier
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        write!(f, "{}@{}", self.uid(), self.fqdn())
    }
}
