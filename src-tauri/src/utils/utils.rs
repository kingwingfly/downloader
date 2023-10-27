use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};
use tempdir::TempDir;

#[cfg(test)]
use std::io::{self, Read};
#[cfg(test)]
use tracing::{debug, instrument, Level};

use crate::config::get_config;

use super::error::TempDirResult;

// region TempDir

pub struct TempDirHandler {
    temp_dir: TempDir,
    v_p: PathBuf,
    a_p: PathBuf,
    o_p: PathBuf,
}

impl Drop for TempDirHandler {
    fn drop(&mut self) {
        #[cfg(test)]
        debug!("merging");
        let statu = std::process::Command::new("/usr/local/bin/ffmpeg")
            .args([
                "-y",
                "-i",
                &self.v_p.to_string_lossy().to_string(),
                "-i",
                &self.a_p.to_string_lossy().to_string(),
                "-c:v",
                "copy",
                "-c:a",
                "copy",
                &self.o_p.to_string_lossy().to_string(),
            ])
            .status();
        #[cfg(test)]
        debug!("temp dir dropped");
    }
}

impl TempDirHandler {
    pub fn new<S: AsRef<str>>(filename: S) -> TempDirResult<Self> {
        let temp_dir = TempDir::new("downloader")?;
        let v_p = temp_dir.path().join(format!("{}.mp4", filename.as_ref()));
        let a_p = temp_dir.path().join(format!("{}.aac", filename.as_ref()));
        let o_p = if cfg!(test) {
            temp_dir.path().join("merge_test.mp4")
        } else {
            Path::new(&get_config("save_dir").unwrap()).join(format!("{}.mp4", filename.as_ref()))
        };
        // TODO illegal filename filter
        Ok(Self {
            temp_dir,
            v_p,
            a_p,
            o_p,
        })
    }

    pub fn write<Su: AsRef<str>>(&self, suffix: Su, buf: &[u8]) -> TempDirResult<()> {
        match suffix.as_ref() {
            ".mp4" | "mp4" => {
                let mut f = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&self.v_p)?;
                f.write_all(buf)?;
                f.sync_all()?;
            }
            ".aac" | "aac" => {
                let mut f = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&self.a_p)?;
                f.write_all(buf)?;
                f.sync_all()?;
            }
            _ => {}
        }
        Ok(())
    }

    pub fn move_<P1, P2>(&self, filename: P1, to: P2) -> std::io::Result<()>
    where
        P1: AsRef<Path>,
        P2: AsRef<Path>,
    {
        let to = to.as_ref().join(&filename);
        let from = self.temp_dir.path().join(filename);
        std::fs::rename(from, to)?;
        Ok(())
    }

    #[cfg(test)]
    #[instrument(level=Level::DEBUG, skip(self), err)]
    pub fn read(&self, filename: &str) -> io::Result<String> {
        let file_path = self.temp_dir.path().join(filename);
        let mut f = std::fs::OpenOptions::new().read(true).open(file_path)?;
        let mut buf = String::new();
        f.read_to_string(&mut buf)?;
        Ok(buf)
    }
}

// endregion TempDir

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[ignore = "don't handle txt"]
    fn temp_dir_test() {
        let temp_file_handler = TempDirHandler::new("test").unwrap();
        assert!(temp_file_handler.write("txt", b"Hello, ").is_ok());
        assert!(temp_file_handler.write("txt", b"world!").is_ok());
        assert_eq!(
            temp_file_handler.read("test.txt").ok(),
            Some("Hello, world!".to_string())
        );
    }

    #[test]
    #[ignore = "don't handle txt"]
    fn move_test() {
        let temp_file_handler = TempDirHandler::new("test").unwrap();
        assert!(temp_file_handler.write(".txt", b"Hello, world").is_ok());
        assert!(temp_file_handler.move_(".txt", "/Users/louis").is_ok());
        assert!(std::fs::remove_file("/Users/louis/test.txt").is_ok());
    }
}
