use config::ConfigError;
use config::{Config, Map, Source, Value, ValueKind};
use rand::rngs::ThreadRng;
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey};
use std::{collections::HashSet, sync::OnceLock};

static APP_CONFIG: OnceLock<Result<Config, ConfigError>> = OnceLock::new();
const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.1 Safari/605.1.15";

pub fn config_init() -> Result<(), ConfigError> {
    APP_CONFIG.get_or_init(|| {
        Config::builder()
            .set_default("user-agent", USER_AGENT)?
            .set_default("bili_cookie", "")?
            .add_source(KeySource)
            .build()
    });
    Ok(())
}

pub fn get_config<S: AsRef<str>>(key: S) -> Option<String> {
    APP_CONFIG
        .get()?
        .as_ref()
        .ok()?
        .get_string(key.as_ref())
        .ok()
}

#[derive(Debug, Clone)]
struct KeySource;

impl Source for KeySource {
    fn clone_into_box(&self) -> Box<dyn Source + Send + Sync> {
        Box::new(self.to_owned())
    }
    fn collect(&self) -> Result<config::Map<String, config::Value>, config::ConfigError> {
        if cfg!(test) {
            dotenv::dotenv().ok();
            Ok(MapWrapper::from(HashSet::from([
                (
                    "bili_cookie",
                    std::env::var("BILI_COOKIE").unwrap_or("".to_string()),
                ),
                ("save_dir", std::env::var("SAVE_DIR").unwrap()),
            ]))
            .0)
        } else {
            Ok(MapWrapper::from(HashSet::from([
                ("bili_cookie", ""),
                ("save_dir", "/Users/louis"),
            ]))
            .0)
        }
    }
}

struct MapWrapper(Map<String, Value>);

impl<K, V> From<HashSet<(K, V)>> for MapWrapper
where
    K: AsRef<str>,
    V: Into<ValueKind>,
{
    fn from(map: HashSet<(K, V)>) -> Self {
        let mut ret = config::Map::new();
        let mut insert = |k: &str, v: V| {
            ret.insert(k.to_owned(), config::Value::new(None, v.into()));
        };
        map.into_iter().for_each(|(k, v)| insert(k.as_ref(), v));
        MapWrapper(ret)
    }
}

fn encrypt<K, V>(
    origin: &HashSet<(K, V)>,
    rng: &mut ThreadRng,
    pub_key: RsaPublicKey,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync + 'static>>
where
    K: AsRef<str> + serde::Serialize,
    V: Into<ValueKind> + AsRef<str> + serde::Serialize,
{
    let origin = serde_json::to_string(origin).unwrap();
    let encrypted = pub_key.encrypt(rng, Pkcs1v15Encrypt, origin.as_bytes())?;
    Ok(encrypted)
}

fn decrypt<K, V>(
    encrypted: Vec<u8>,
    priv_key: RsaPrivateKey,
) -> Result<HashSet<(K, V)>, Box<dyn std::error::Error + Send + Sync + 'static>>
where
    for<'de> K: AsRef<str> + serde::Deserialize<'de> + std::cmp::Eq + std::hash::Hash,
    for<'de> V:
        Into<ValueKind> + AsRef<str> + serde::Deserialize<'de> + std::cmp::Eq + std::hash::Hash,
{
    let decrypted = priv_key.decrypt(Pkcs1v15Encrypt, &encrypted)?;
    let origin = serde_json::from_str(std::str::from_utf8(&decrypted)?)?;
    Ok(origin)
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn init_config_test() {
        assert!(config_init().is_ok());
        assert_eq!(get_config("user-agent").unwrap(), USER_AGENT);
        assert_ne!(get_config("cookie").unwrap(), "")
    }

    #[test]
    fn encrypt_test() {
        let mut rng = rand::thread_rng();
        let bits = 2048;
        let priv_key = RsaPrivateKey::new(&mut rng, bits).expect("failed to generate a key");
        let pub_key = RsaPublicKey::from(&priv_key);
        let orgin = HashSet::from([("1".to_string(), "1".to_string())]);
        let encrypted = encrypt(&orgin, &mut rng, pub_key);
        assert!(encrypted.is_ok());
        let encrypted = encrypted.unwrap();
        let decrypted = decrypt(encrypted, priv_key);
        assert!(decrypted.is_ok());
        assert_eq!(decrypted.unwrap(), orgin);
    }
}
