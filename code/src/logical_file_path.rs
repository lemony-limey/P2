use log::debug;
use std::hash::{Hash, Hasher};
use std::fmt;
use std::fmt::Formatter;

/// Logical file paths represent the file path relative to the logical root directory, and not the
/// local unix root directory.
/// They do not contain the root directory itself.
/// For example, "/" refers to "/{path_to_root_dir}/", and so "/dir1/hello.txt" refers to
/// "/{path_to_root_dir}/dir1/hello.txt"
#[derive(Clone, Debug)]
pub struct LogicalFilePath(String);

impl LogicalFilePath
{
    pub fn new(logical_path: impl ToString) -> Self
    {
        let mut logical_path = logical_path.to_string();
        if !logical_path.starts_with("/")
        {
            debug!("logical path \"{}\" requires correction", &logical_path);
            let mut logical_path_corrected = String::from("/");
            logical_path_corrected.push_str(&logical_path);
            logical_path = logical_path_corrected;
        }

        debug!("logical path: \"{}\"", logical_path);
        Self(logical_path.to_string())
    }

    pub fn get(&self) -> &String
    {
        &self.0
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
