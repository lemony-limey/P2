use crate::protocol::message::ResponseID;
use std::fmt::{self, Formatter};

// R3 - Download
#[derive(Clone, Debug)]
pub enum DownloadMessage
{
    Request
    {
        file_string: String,
    },
    Result
    {
        response_id: ResponseID,
        file_string: String,
    },
}

impl fmt::Display for DownloadMessage
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        write!(f, "download:")?;

        match self
        {
            DownloadMessage::Request { file_string } => {
                write!(f, "request:{}", file_string)
            }
            DownloadMessage::Result { response_id, file_string } => {
                // write!(f, "result:{}:{}", response_id, file_string)
                todo!()
            }
        }
    }
}
