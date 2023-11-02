use config::ConfigError;
use config::{Config, Source, Value, ValueKind};
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey};
use std::collections::HashMap;
use std::{collections::HashSet, sync::OnceLock};

static APP_CONFIG: OnceLock<Result<Config, ConfigError>> = OnceLock::new();
const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.1 Safari/605.1.15";

pub fn config_init() -> Result<(), ConfigError> {
    APP_CONFIG.get_or_init(|| {
        Config::builder()
            .set_default("user-agent", USER_AGENT)?
            .set_default("bili_cookie", "")?
            .add_source(KeySource::new())
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
struct KeySource {
    inner: HashMap<String, Value>,
}

impl KeySource {
    #[cfg(test)]
    fn new() -> Self {
        dotenv::dotenv().ok();
        HashSet::from([
            (
                "bili_cookie",
                std::env::var("BILI_COOKIE").unwrap_or("".to_string()),
            ),
            ("save_dir", std::env::var("SAVE_DIR").unwrap()),
        ])
        .into()
    }

    #[cfg(not(test))]
    fn new() -> Self {
        HashSet::from([("save_dir", "/Users/louis")]).into()
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

#[derive(serde::Serialize, serde::Deserialize)]
struct Encrypter {
    priv_key: RsaPrivateKey,
    pub_key: RsaPublicKey,
}

impl Encrypter {
    fn new() -> Self {
        let mut rng = rand::thread_rng();
        let bits = 2048;
        let priv_key = RsaPrivateKey::new(&mut rng, bits).expect("failed to generate a key");
        let pub_key = RsaPublicKey::from(&priv_key);
        Self { priv_key, pub_key }
    }

    fn from_key_ring() -> Self {
        let entry = key_ring_entry();
        match entry.get_password() {
            Ok(serded_enc) => serde_json::from_str(&serded_enc).unwrap(),
            _ => {
                let new_enc = Encrypter::new();
                entry
                    .set_password(&serde_json::to_string(&new_enc).unwrap())
                    .unwrap();
                new_enc
            }
        }
    }

    fn encrypt<I>(
        &self,
        origin: &I,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync + 'static>>
    where
        I: serde::Serialize,
    {
        let mut rng = rand::thread_rng();
        let origin = serde_json::to_vec(origin).unwrap();
        let encrypted = self.pub_key.encrypt(&mut rng, Pkcs1v15Encrypt, &origin)?;
        Ok(encrypted)
    }

    fn decrypt<R>(
        &self,
        encrypted: &[u8],
    ) -> Result<R, Box<dyn std::error::Error + Send + Sync + 'static>>
    where
        for<'de> R: serde::Deserialize<'de>,
    {
        let decrypted = self.priv_key.decrypt(Pkcs1v15Encrypt, &encrypted)?;
        let origin = serde_json::from_slice(&decrypted)?;
        Ok(origin)
    }
}

fn key_ring_entry() -> keyring::Entry {
    let user = std::env::var("USER").unwrap_or("downloader user".to_string());
    keyring::Entry::new_with_target("user", "downloader", &user).unwrap()
}

#[cfg(test)]
impl Drop for Encrypter {
    fn drop(&mut self) {
        key_ring_entry().delete_password().ok();
    }
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
        let encrypter = Encrypter::new();
        let orgin = HashSet::from([("1".to_string(), "1".to_string())]);
        let encrypted = encrypter.encrypt(&orgin);
        assert!(encrypted.is_ok());
        let encrypted = encrypted.unwrap();
        let decrypted = encrypter.decrypt::<HashSet<(String, String)>>(&encrypted);
        assert!(decrypted.is_ok());
        assert_eq!(decrypted.unwrap(), orgin);
    }

    #[test]
    fn encrtpter_create_test() {
        keyring::set_default_credential_builder(keyring::mock::default_credential_builder());
        let e1 = Encrypter::from_key_ring();
        let e2 = Encrypter::from_key_ring();
        let data = HashMap::from([("hello", "world")]);
        let enc_ret1 = e1.encrypt(&data).unwrap();
        let enc_ret2 = e2.encrypt(&data).unwrap();
        assert_ne!(enc_ret1, enc_ret2);
        assert_eq!(
            e1.decrypt::<HashMap<String, String>>(&enc_ret1).unwrap(),
            e2.decrypt::<HashMap<String, String>>(&enc_ret2).unwrap()
        );
    }
}
