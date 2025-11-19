use crate::protocol::identifier::Identifier;
use crate::protocol::message::ResponseID;
use std::fmt::{self, Formatter};
use crate::logical_file_path::LogicalFilePath;

// R3 - Download
#[derive(Clone, Debug)]
pub enum DownloadMessage
{
    Request
    {
        identifier: Identifier,
        file_path: LogicalFilePath,
    },
    Result
    {
        response_id: ResponseID,
        file_path: LogicalFilePath,
    },
}

impl fmt::Display for DownloadMessage
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        write!(f, "download:")?;

        match self
        {
            DownloadMessage::Request { identifier, file_path: file_string } => {
                write!(f, "request:{}:{}", identifier, file_string)
            }
            DownloadMessage::Result { response_id, file_path: file_string } => {
                write!(f, "result:{}:{}:complete", response_id, file_string)
            }
        }
    }
}
