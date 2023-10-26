use std::{io::Write, path::Path};
use tempdir::TempDir;

#[cfg(test)]
use std::io::{self, Read};
#[cfg(test)]
use tracing::{debug, instrument, Level};

use crate::config::get_config;

use super::error::TempDirResult;

// region TempDir

pub struct TempDirHandler<S>
where
    S: 'static + AsRef<str> + Send + Sync,
{
    temp_dir: TempDir,
    filename: S,
}

impl<S> Drop for TempDirHandler<S>
where
    S: 'static + AsRef<str> + Send + Sync,
{
    fn drop(&mut self) {
        #[cfg(test)]
        debug!("merging and move");
        let v_p = self
            .temp_dir
            .path()
            .join(format!("{}.mp4", self.filename.as_ref()))
            .to_string_lossy()
            .to_string();
        let a_p = self
            .temp_dir
            .path()
            .join(format!("{}.aac", self.filename.as_ref()))
            .to_string_lossy()
            .to_string();
        let o_p = Path::new(&get_config("save_dir").unwrap())
            .join(format!("{}.mp4", self.filename.as_ref()))
            .to_string_lossy()
            .to_string();
        println!("{}\n{}\n{}", v_p, a_p, o_p);

        let output = std::process::Command::new("/usr/local/bin/ffmpeg")
            .args([
                "-y", "-i", &v_p, "-i", &a_p, "-c:v", "copy", "-c:a", "copy", &o_p,
            ])
            .output()
            .unwrap();
        std::io::stdout().write_all(&output.stdout).unwrap();
        std::io::stderr().write_all(&output.stderr).unwrap();
        #[cfg(test)]
        debug!("temp dir dropped");
    }
}

impl<S> TempDirHandler<S>
where
    S: 'static + AsRef<str> + Send + Sync,
{
    pub fn new(filename: S) -> TempDirResult<Self> {
        // TODO illegal filename filter
        Ok(Self {
            temp_dir: TempDir::new("downloader")?,
            filename,
        })
    }

    pub fn write<Su: AsRef<str>>(&self, suffix: Su, buf: &[u8]) -> TempDirResult<()> {
        let file_path = self.temp_dir.path().join(format!(
            "{}.{}",
            self.filename.as_ref(),
            suffix.as_ref().replace(".", "")
        ));
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_path)?;
        f.write_all(buf)?;
        f.sync_all()?;
        // #[cfg(test)]
        // debug!("{} bytes writed to {:?}", buf.len(), file_path);
        Ok(())
    }

    pub fn move_<P1, P2>(&self, filename: P1, to: P2) -> std::io::Result<()>
    where
        P1: AsRef<Path>,
        P2: AsRef<Path>,
    {
        let to = to.as_ref().join(&filename);
        let from = self.temp_dir.path().join(filename);
        std::fs::rename(&from, &to)?;
        #[cfg(test)]
        debug!("move from {:?} to {:?}", from, to);
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
    fn move_test() {
        let temp_file_handler = TempDirHandler::new("test").unwrap();
        assert!(temp_file_handler.write(".txt", b"Hello, world").is_ok());
        assert!(temp_file_handler.move_(".txt", "/Users/louis").is_ok());
        assert!(std::fs::remove_file("/Users/louis/test.txt").is_ok());
    }
}
