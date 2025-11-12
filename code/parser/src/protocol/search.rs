use crate::protocol::message::ResponseID;
use std::fmt::{self, Formatter};


// R2 - Search
#[derive(Clone, Debug)]
pub enum SearchMessage<'a>
{
    Request
    {
        search_string: String,
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

impl fmt::Display for SearchMessage<'_>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        write!(f, "search:")?;

        match self
        {
            SearchMessage::Request{ search_string } => {
                write!(f, "request:{}", search_string)
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