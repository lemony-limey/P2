use std::fmt::{
    self,
    Formatter,
};

// R3 - Download
#[derive(Clone, Debug)]
pub enum DownloadMessage
{
    Request
    {

    },
    Result
    {

    },
    Error
    {

    },
}

impl fmt::Display for DownloadMessage
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        write!(f, "download:")?;

        match self
        {
            DownloadMessage::Request { .. } => {
                todo!()
            }
            DownloadMessage::Result { .. } => {
                todo!()
            }
            DownloadMessage::Error { .. } => {
                todo!()
            }
        }
    }
}
