mod encrypt;

use config::ConfigError;
use config::{Config, Source, Value, ValueKind};
use std::collections::HashMap;
use std::{collections::HashSet, sync::OnceLock};

static APP_CONFIG: OnceLock<Config> = OnceLock::new();
const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.1 Safari/605.1.15";

pub fn config_init() -> Result<(), ConfigError> {
    let config = Config::builder()
        .set_default("user-agent", USER_AGENT)?
        .set_default("save_dir", dirs_next::download_dir().unwrap().to_str())?
        .set_default("bili_cookie", "")?
        .add_source(KeySource::new())
        .build()?;
    APP_CONFIG.set(config).unwrap();
    Ok(())
}

pub fn get_config<S: AsRef<str>>(key: S) -> Option<String> {
    APP_CONFIG.get()?.get_string(key.as_ref()).ok()
}

#[derive(Debug, Clone, Default)]
struct KeySource {
    inner: HashMap<String, Value>,
}

impl KeySource {
    #[cfg(test)]
    fn test_new() -> Self {
        let config_dir = dirs_next::config_dir().unwrap().join("downloader");
        std::fs::create_dir_all(&config_dir).unwrap();
        tracing::debug!("{:?}", config_dir);
        dotenv::dotenv().ok();
        let hs = HashMap::from([
            (
                "bili_cookie",
                std::env::var("BILI_COOKIE").unwrap_or("".to_string()),
            ),
            ("save_dir", std::env::var("SAVE_DIR").unwrap()),
        ]);
        let key_source: KeySource = hs.into();
        key_source.save();
        key_source
    }

    fn new() -> Self {
        let config_dir = dirs_next::config_dir().unwrap().join("downloader");
        std::fs::create_dir_all(&config_dir).unwrap();
        let mut ret = HashMap::new();
        let encrypter = encrypt::Encrypter::from_key_ring();
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
        ret.into()
    }

    fn save(&self) {
        let config_dir = dirs_next::config_dir().unwrap().join("downloader");
        std::fs::create_dir_all(&config_dir).unwrap();
        let encrypter = encrypt::Encrypter::from_key_ring();
        for (filename, data) in self.inner.iter() {
            let encrypted = encrypter.encrypt(&data.to_string()).unwrap();
            std::fs::write(config_dir.join(format!("{filename}_new")), &encrypted).unwrap();
            std::fs::rename(
                config_dir.join(format!("{filename}_new")),
                config_dir.join(filename),
            )
            .unwrap();
        }
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
        KeySource::test_new();
        assert!(config_init().is_ok());
        assert_eq!(get_config("user-agent").unwrap(), USER_AGENT);
        assert_ne!(get_config("bili_cookie").unwrap(), "")
    }
}
