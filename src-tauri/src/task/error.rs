use reqwest::Error as ReqwestError;
use snafu::prelude::*;

#[derive(Debug, Snafu)]
#[snafu(module, visibility(pub(crate)), context(suffix(Error)))]
pub enum TaskError {
    #[snafu(display("Could not parse url"), context(false))]
    ParseUrl { source: url::ParseError },
    #[snafu(display("Maybe network disconnected"), context(false))]
    GetError { source: ReqwestError },
    #[snafu(context(false))]
    ParseHtmlError { source: ParseError },
    #[snafu()]
    StatusError,
    #[snafu(context(suffix(false)))]
    UnknownTaskType,
    #[snafu(context(suffix(false)))]
    ConfigNotFound,
    #[snafu(context(false))]
    SaveError { source: ActorError },
    #[snafu(context(false))]
    ActixError { source: actix::MailboxError },
}

pub type TaskResult<T> = Result<T, TaskError>;

#[derive(Debug, Snafu)]
#[snafu(module, visibility(pub(crate)), context(suffix(Error)))]
pub enum ParseError {
    #[snafu(context(suffix(false)))]
    NoTargetFound,
    #[snafu()]
    JsonParseError { source: serde_json::error::Error },
    #[snafu(context(suffix(false)))]
    BiliPlayInfoNotFound,
}

pub type ParseResult<T> = Result<T, ParseError>;

#[derive(Debug, Snafu)]
#[snafu(module, visibility(pub(crate)), context(suffix(Error)))]
pub enum ActorError {
    #[snafu()]
    BiliSaveError,
    #[snafu(context(suffix(false)))]
    ContentLengthUnknown,
    #[snafu()]
    NetWorkError { source: reqwest::Error },
    #[snafu(context(suffix(false)))]
    Cancelled,
}

pub type ActorResult<T> = Result<T, ActorError>;
