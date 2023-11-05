mod encrypt;
pub mod error;

use config::{Config, Source, Value, ValueKind};
use error::ConfigResult;
use std::collections::HashMap;
use std::{collections::HashSet, sync::OnceLock};

use error::config_error;
use snafu::OptionExt;

static mut APP_CONFIG: OnceLock<Config> = OnceLock::new();
const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.1 Safari/605.1.15";

pub fn config_init() -> ConfigResult<()> {
    unsafe {
        APP_CONFIG.take();
        let config = Config::builder()
            .set_default("user-agent", USER_AGENT)?
            .set_default(
                "save_dir",
                dirs_next::download_dir()
                    .context(config_error::ConfigDirUnknown)?
                    .to_str(),
            )?
            .set_default("bili_cookie", "")?
            .add_source(KeySource::new()?)
            .build()?;
        APP_CONFIG.set(config).unwrap();
    }
    Ok(())
}

pub fn get_config<S: AsRef<str>>(key: S) -> Option<String> {
    config_init().ok()?;
    unsafe { APP_CONFIG.get()?.get_string(key.as_ref()).ok() }
}

pub fn upgrade_config<KV>(kv: KV) -> ConfigResult<()>
where
    KV: Into<KeySource>,
{
    let mut keysource = KeySource::new()?;
    keysource.upgrade(kv)?;
    Ok(())
}

pub fn show_config() -> Option<HashMap<String, String>> {
    config_init().ok()?;
    unsafe {
        Some(
            APP_CONFIG
                .get()?
                .collect()
                .ok()?
                .into_iter()
                .map(|(k, v)| (k, v.to_string()))
                .collect(),
        )
    }
}

#[derive(Debug, Clone, Default)]
pub struct KeySource {
    inner: HashMap<String, Value>,
}

impl KeySource {
    #[cfg(test)]
    fn init() -> Self {
        let config_dir = dirs_next::config_dir().unwrap().join("downloader");
        std::fs::create_dir_all(&config_dir).unwrap();
        tracing::debug!("{:?}", config_dir);
        dotenv::dotenv().ok();
        let hs = HashMap::from([(
            "bili_cookie",
            std::env::var("BILI_COOKIE").unwrap_or("".to_string()),
        )]);
        let key_source: KeySource = hs.into();
        key_source.save().unwrap();
        key_source
    }

    fn new() -> ConfigResult<Self> {
        let config_dir = dirs_next::config_dir()
            .context(config_error::ConfigDirUnknown)?
            .join("downloader");
        std::fs::create_dir_all(&config_dir).unwrap();
        let mut ret = HashMap::new();
        let encrypter = encrypt::Encrypter::from_key_ring()?;
        let entry = std::fs::read_dir(&config_dir).unwrap();
        for path in entry.filter_map(|p| p.map(|p| p.path()).ok()) {
            match encrypter.decrypt::<String>(&std::fs::read(&path).unwrap()) {
                Ok(data) => {
                    ret.insert(
                        path.file_name().unwrap().to_string_lossy().to_string(),
                        data,
                    );
                }
                Err(_) => {}
            }
        }
        Ok(ret.into())
    }

    fn upgrade<KV>(&mut self, kv: KV) -> ConfigResult<()>
    where
        KV: Into<KeySource>,
    {
        let key_source: KeySource = kv.into();
        self.inner.extend(key_source.inner);
        self.save()?;
        Ok(())
    }

    fn save(&self) -> ConfigResult<()> {
        let config_dir = dirs_next::config_dir()
            .context(config_error::ConfigDirUnknown)?
            .join("downloader");
        std::fs::create_dir_all(&config_dir).unwrap();
        let encrypter = encrypt::Encrypter::from_key_ring()?;
        for (filename, data) in self.inner.iter() {
            let encrypted = encrypter.encrypt(&data.to_string()).unwrap();
            std::fs::write(config_dir.join(format!("{filename}_new")), &encrypted).unwrap();
            std::fs::rename(
                config_dir.join(format!("{filename}_new")),
                config_dir.join(filename),
            )
            .unwrap();
        }
        Ok(())
    }
}

impl Source for KeySource {
    fn clone_into_box(&self) -> Box<dyn Source + Send + Sync> {
        Box::new(self.to_owned())
    }

    fn collect(&self) -> Result<config::Map<String, config::Value>, config::ConfigError> {
        Ok(self.inner.to_owned())
    }
}

impl<K, V> Into<KeySource> for HashSet<(K, V)>
where
    K: AsRef<str>,
    V: Into<ValueKind>,
{
    fn into(self) -> KeySource {
        let inner = self
            .into_iter()
            .map(|(k, v)| (k.as_ref().to_owned(), config::Value::new(None, v.into())))
            .collect();
        KeySource { inner }
    }
}

impl<K, V> Into<KeySource> for HashMap<K, V>
where
    K: AsRef<str>,
    V: Into<ValueKind>,
{
    fn into(self) -> KeySource {
        let inner = self
            .into_iter()
            .map(|(k, v)| (k.as_ref().to_owned(), config::Value::new(None, v.into())))
            .collect();
        KeySource { inner }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tracing_test::traced_test]
    #[test]
    fn init_config_test() {
        KeySource::init();
        dotenv::dotenv().ok();
        assert!(config_init().is_ok());
        assert_eq!(get_config("user-agent").unwrap(), USER_AGENT);
        assert_eq!(
            get_config("bili_cookie").unwrap_or("".to_string()),
            std::env::var("BILI_COOKIE").unwrap()
        )
    }

    #[test]
    fn upgrade_config_test() {
        let mut keysource = KeySource::new().unwrap();
        keysource
            .upgrade(HashSet::from([("hello", "world")]))
            .unwrap();
        assert_eq!(keysource.inner["hello"].to_string(), "world");
        let keysource = KeySource::new().unwrap();
        assert_eq!(keysource.inner["hello"].to_string(), "world");
    }

    #[test]
    fn show_config_test() {
        let hm = show_config().unwrap();
        println!("{:#?}", hm);
    }
}
