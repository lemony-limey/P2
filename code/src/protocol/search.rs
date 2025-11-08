use std::fmt;
use std::fmt::Formatter;
use crate::ResponseID;

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
        file_string: String,
    },
    Error
    {
        response_id: ResponseID,
    },
}

impl fmt::Display for SearchMessage
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        write!(f, "search:")?;

        match self
        {
            SearchMessage::Request(request) => {
                write!(f, "request:{}", request)
            }
            SearchMessage::Result { response_id, file_string } => {
                write!(f, "result:{}:{}", response_id, file_string)
            }
            SearchMessage::Error { response_id } => {
                write!(f, "error:{}", response_id)
            }
        }
    }
}