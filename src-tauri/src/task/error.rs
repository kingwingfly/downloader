use reqwest::Error as ReqwestError;
use snafu::prelude::*;

#[derive(Debug, Snafu)]
#[snafu(module, visibility(pub(crate)), context(suffix(Error)))]
pub enum TaskError {
    #[snafu(display("Could not parse url: {url}"))]
    ParseUrl {
        source: url::ParseError,
        url: String,
    },
    #[snafu(context(false))]
    GetError { source: ReqwestError },
    #[snafu(context(false))]
    ParseHtmlError { source: ParseError },
    #[snafu()]
    StatusError,
    #[snafu(context(suffix(false)))]
    UnknownTaskType,
}

pub type TaskResult<T> = Result<T, TaskError>;

#[derive(Debug, Snafu)]
#[snafu(module, visibility(pub(crate)), context(suffix(Error)))]
pub enum ParseError {
    #[snafu(context(suffix(false)))]
    NoTargetFound,
    #[snafu()]
    JsonParseError,
}

pub type ParseResult<T> = Result<T, ParseError>;
