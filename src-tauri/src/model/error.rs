use snafu::prelude::*;
use uuid::Uuid;

use crate::task::TaskError;

#[derive(Debug, Snafu)]
#[snafu(module, visibility(pub(crate)), context(suffix(Error)))]
pub enum BmcError {
    #[snafu(display("Cannot createc task for error: {}", source), context(false))]
    NewTaskError { source: TaskError },
    #[snafu(display("Task id not found: {}", id))]
    TaskNotFound { id: Uuid },
}

pub type BmcResult<T> = Result<T, BmcError>;
