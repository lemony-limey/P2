use std::fmt::{
    self,
    Formatter,
};

// R3 - Download
#[derive(Clone, Debug)]
pub enum DownloadMessage
{
    DownloadRequest
    {

    },
    DownloadResult
    {

    },
    DownloadError
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
            DownloadMessage::DownloadRequest { .. } => {
                todo!()
            }
            DownloadMessage::DownloadResult { .. } => {
                todo!()
            }
            DownloadMessage::DownloadError { .. } => {
                todo!()
            }
        }
    }
}
