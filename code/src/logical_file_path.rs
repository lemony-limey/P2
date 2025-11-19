use std::hash::{Hash, Hasher};
use std::fmt;
use std::fmt::Formatter;

/// Logical file paths represent the file path relative to the logical root directory, and not the
/// local unix root directory.
/// They do not contain the root directory itself.
/// For example, "/" refers to "/{path_to_root_dir}/", and so "/dir1/hello.txt" refers to
/// "/{path_to_root_dir}/dir1/hello.txt"
#[derive(Clone, Debug)]
pub struct LogicalFilePath(pub String);

impl LogicalFilePath
{
    /// TODO: Should this be a fallible conversion? Probably not.
    pub fn new(logical_path: impl ToString) -> Self
    {
        Self(logical_path.to_string())
    }
}

impl fmt::Display for LogicalFilePath
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        write!(f, "{}", self.0)
    }
}

impl PartialEq for LogicalFilePath
{
    fn eq(&self, other: &Self) -> bool
    {
        self.0.eq(&other.0)
    }
}

impl PartialEq<String> for LogicalFilePath
{
    fn eq(&self, other: &String) -> bool
    {
        self.0.eq(other)
    }
}

impl Eq for LogicalFilePath {}

impl Hash for LogicalFilePath
{
    fn hash<H: Hasher>(&self, state: &mut H)
    {
        self.0.hash(state)
    }
}
