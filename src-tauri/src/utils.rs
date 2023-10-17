use std::io::Write;
use tempdir::TempDir;
use tracing::debug;

use crate::error::TaskResult;

#[cfg(test)]
use std::io::{self, Read};
#[cfg(test)]
use tracing::{instrument, Level};

pub struct TempDirHandler {
    temp_dir: TempDir,
}

impl TempDirHandler {
    pub fn new() -> TaskResult<Self> {
        Ok(Self {
            temp_dir: TempDir::new("downloader")?,
        })
    }

    pub fn write(&self, filename: &str, buf: &[u8]) -> TaskResult<()> {
        let file_path = self.temp_dir.path().join(filename);
        debug!("Write temp file: {:?}", file_path);
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_path)?;
        f.write_all(buf)?;
        f.sync_all()?;
        Ok(())
    }

    #[cfg(test)]
    #[instrument(level=Level::DEBUG, skip(self), err)]
    pub fn read(&self, filename: &str) -> io::Result<String> {
        let file_path = self.temp_dir.path().join(filename);
        debug!("Read temp file: {:?}", file_path);
        let mut f = std::fs::OpenOptions::new().read(true).open(file_path)?;
        let mut buf = String::new();
        f.read_to_string(&mut buf)?;
        Ok(buf)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn temp_dir_test() {
        let temp_file_handler = TempDirHandler::new().unwrap();
        assert!(temp_file_handler.write("test.txt", b"Hello, ").is_ok());
        assert!(temp_file_handler.write("test.txt", b"world!").is_ok());
        assert_eq!(
            temp_file_handler.read("test.txt").ok(),
            Some("Hello, world!".to_string())
        );
    }
}
