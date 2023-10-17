use std::io;

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
    #[snafu(display("{}", source), context(false))]
    GetError {
        source: ReqwestError,
    },
    #[snafu(
        display("Could not GET url: {:?}", std::env::temp_dir()),
        context(false)
    )]
    IoError {
        source: io::Error,
    },
    StatusError,
    UnknownTaskType,
}

pub type TaskResult<T> = Result<T, TaskError>;
