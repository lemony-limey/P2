use crate::protocol::message::ResponseID;
use std::fmt::{self, Formatter};

// R3 - Download
#[derive(Clone, Debug)]
pub enum DownloadMessage<'a>
{
    Request
    {
        file_string: String,
    },
    Result
    {
        response_id: ResponseID<'a>,
        file_string: String,
    },
    Error
    {
        response_id: ResponseID<'a>,
    },
}

impl fmt::Display for DownloadMessage<'_>
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
            DownloadMessage::Error { response_id } => {
                // write!(f, "error:{}", response_id)
                todo!()
            }
        }
    }
}
