use std::fmt;
use std::fmt::Formatter;
use crate::ResponseID;

// R2 - Search
#[derive(Clone, Debug)]
pub enum SearchMessage
{
    SearchRequest(String),
    SearchResult
    {
        response_id: ResponseID,
        file_string: String,
    },
    SearchError
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
            SearchMessage::SearchRequest(request) => {
                write!(f, "request:{}", request)
            }
            SearchMessage::SearchResult { response_id, file_string } => {
                write!(f, "result:{}:{}", response_id, file_string)
            }
            SearchMessage::SearchError { response_id } => {
                write!(f, "error:{}", response_id)
            }
        }
    }
}