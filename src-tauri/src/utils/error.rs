use std::io;

use snafu::prelude::*;

#[derive(Debug, Snafu)]
#[snafu(module, visibility(pub(crate)), context(suffix(Error)))]
pub enum TemDirError {
    #[snafu(
        display("Could not create tempdir: {}", std::env::temp_dir().display()),
        context(false)
    )]
    IoError { source: io::Error },
}

pub type TempDirResult<T> = Result<T, TemDirError>;
