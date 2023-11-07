mod error;

use std::{
    io::Write,
    path::{Path, PathBuf},
};
use tempdir::TempDir;

#[cfg(test)]
use std::io::{self, Read};
#[cfg(test)]
use tracing::{debug, instrument, Level};

use crate::config::get_config;

use error::TempDirResult;

// region TempDir

#[cfg_attr(test, derive(Debug))]
pub struct TempDirHandler {
    temp_dir: TempDir,
    filename: String,
    o_p: PathBuf,
}

#[cfg(test)]
impl Drop for TempDirHandler {
    fn drop(&mut self) {
        debug!("TempDirHandler dropping");
    }
}

impl TempDirHandler {
    pub fn new<S: AsRef<str>>(filename: S) -> TempDirResult<Self> {
        let filename = sanitize_filename::sanitize(filename);
        let temp_dir = TempDir::new("downloader")?;
        let o_p = if cfg!(test) {
            temp_dir.path().to_path_buf()
        } else {
            Path::new(&get_config("save_dir").unwrap()).to_path_buf()
        };
        std::fs::create_dir_all(&o_p).unwrap();
        Ok(Self {
            temp_dir,
            filename,
            o_p,
        })
    }

    pub fn write<Su: AsRef<str>>(&self, suffix: Su, buf: &[u8]) -> TempDirResult<()> {
        let path = self
            .temp_dir
            .path()
            .join(format!("{}.{}", self.filename, suffix.as_ref()));
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        f.write_all(buf)?;
        f.sync_all()?;
        Ok(())
    }

    pub fn save(&self) {
        #[cfg(test)]
        debug!("saving");
        let mut cmd =
            std::process::Command::new(get_config("ffmpeg").unwrap_or("ffmpeg".to_string()));
        for path in std::fs::read_dir(self.temp_dir.path()).unwrap() {
            let path = path.unwrap();
            match new_mime_guess::from_path(path.path()).first() {
                Some(mime) => match mime.type_() {
                    mime::VIDEO | mime::AUDIO => {
                        cmd.args(["-i", path.path().to_string_lossy().as_ref()]);
                    }
                    _ => {
                        self.move_(path.file_name()).ok();
                    }
                },
                None => {}
            }
        }
        let o_p = if cfg!(test) {
            self.o_p.join("merge_test.mp4")
        } else {
            self.o_p.join(format!("{}.mp4", self.filename))
        };
        cmd.args([
            "-y",
            "-c:v",
            "copy",
            "-c:a",
            "copy",
            o_p.to_string_lossy().as_ref(),
        ]);
        let _statu = cmd.status();
    }

    pub fn move_<P>(&self, filename: P) -> std::io::Result<()>
    where
        P: AsRef<Path>,
    {
        let from = self.temp_dir.path().join(filename.as_ref());
        let to = self.o_p.parent().unwrap().join(filename.as_ref());
        #[cfg(test)]
        debug!("move from {:?} to {:?}", from, to);
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
        assert!(temp_file_handler.write("txt", b"Hello, world").is_ok());
        assert!(temp_file_handler.move_("txt").is_ok());
        assert!(std::fs::remove_file("/Users/louis/test.txt").is_ok());
    }
}
