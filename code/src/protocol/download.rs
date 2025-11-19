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
        logical_file_path: LogicalFilePath,
    },
    Result
    {
        response_id: ResponseID,
        logical_file_path: LogicalFilePath,
    },
}

impl fmt::Display for DownloadMessage
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        write!(f, "download:")?;

        match self
        {
            DownloadMessage::Request { identifier, logical_file_path } => {
                write!(f, "request:{}:{}", identifier, logical_file_path)
            }
            DownloadMessage::Result { response_id, logical_file_path } => {
                write!(f, "result:{}:{}:complete", response_id, logical_file_path)
            }
        }
    }
}
