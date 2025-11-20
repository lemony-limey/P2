use crate::protocol::message::ResponseID;
use std::fmt::{self, Formatter};
use crate::logical_file_path::LogicalFilePath;

// R2 - Search
#[derive(Clone, Debug)]
pub enum SearchMessage
{
    Request
    {
        search_string: String,
    },
    Result
    {
        response_id: ResponseID,
        file_path: LogicalFilePath,
        file_size: u64,
    },
}

impl fmt::Display for SearchMessage
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        write!(f, "search:")?;

        match self
        {
            SearchMessage::Request{ search_string } => {
                write!(f, "request:{}", search_string)
            }
            SearchMessage::Result { response_id, file_path, file_size } => {
                write!(f, "result:{}:{}:{}", response_id, file_path, file_size)
            }
        }
    }
}