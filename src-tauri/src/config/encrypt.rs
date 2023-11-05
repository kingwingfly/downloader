use super::error::ConfigResult;
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey};

#[derive(serde::Serialize, serde::Deserialize)]
#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct Encrypter {
    priv_key: RsaPrivateKey,
    #[cfg(not(target_os = "windows"))]
    pub_key: RsaPublicKey,
}

impl Encrypter {
    fn new() -> Self {
        let mut rng = rand::thread_rng();
        let bits = if cfg!(not(target_os = "windows")) {
            2048
        } else {
            1200
        };
        let priv_key = RsaPrivateKey::new(&mut rng, bits).expect("failed to generate a key");
        #[cfg(not(target_os = "windows"))]
        let pub_key = RsaPublicKey::from(&priv_key);
        Self {
            priv_key,
            #[cfg(not(target_os = "windows"))]
            pub_key,
        }
    }

    pub fn from_key_ring() -> ConfigResult<Self> {
        let entry = keyring_entry();
        match entry.get_password() {
            Ok(serded_enc) => Ok(serde_json::from_str(&serded_enc).unwrap()),
            Err(_) => {
                let new_enc = Encrypter::new();
                entry.set_password(&serde_json::to_string(&new_enc).unwrap())?;
                Ok(new_enc)
            }
        }
    }

    pub fn encrypt<I>(&self, origin: &I) -> ConfigResult<Vec<u8>>
    where
        I: serde::Serialize,
    {
        let mut rng = rand::thread_rng();
        let origin = serde_json::to_vec(origin).unwrap();
        #[cfg(not(target_os = "windows"))]
        let encrypted = self.pub_key.encrypt(&mut rng, Pkcs1v15Encrypt, &origin)?;
        #[cfg(target_os = "windows")]
        let encrypted =
            RsaPublicKey::from(&self.priv_key).encrypt(&mut rng, Pkcs1v15Encrypt, &origin)?;
        Ok(encrypted)
    }

    pub fn decrypt<R>(&self, encrypted: &[u8]) -> ConfigResult<R>
    where
        for<'de> R: serde::Deserialize<'de>,
    {
        let decrypted = self.priv_key.decrypt(Pkcs1v15Encrypt, &encrypted)?;
        let origin = serde_json::from_slice(&decrypted)?;
        Ok(origin)
    }
}

fn keyring_entry() -> keyring::Entry {
    let user = std::env::var("USER").unwrap_or("downloader user".to_string());
    keyring::Entry::new_with_target("user", "downloader", &user).unwrap()
}

#[cfg(test)]
mod test {
    use std::collections::{HashMap, HashSet};

    use super::*;

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
        let e1 = Encrypter::from_key_ring().unwrap();
        let e2 = Encrypter::from_key_ring().unwrap();
        assert_eq!(e1, e2);
        let data = HashMap::from([("hello", "world")]);
        let enc_ret1 = e1.encrypt(&data).unwrap();
        let enc_ret2 = e2.encrypt(&data).unwrap();
        assert_ne!(enc_ret1, enc_ret2);
        assert_eq!(
            e1.decrypt::<HashMap<String, String>>(&enc_ret1).unwrap(),
            e2.decrypt::<HashMap<String, String>>(&enc_ret2).unwrap()
        );
    }

    #[test]
    fn encrypter_string_test() {
        let s = "hello world".to_string();
        let encrypter = Encrypter::new();
        let enc_ret = encrypter.encrypt(&s);
        assert!(enc_ret.is_ok());
        let dec_ret = encrypter.decrypt::<String>(&enc_ret.unwrap());
        assert!(dec_ret.is_ok());
        assert_eq!(s, dec_ret.unwrap());
    }
}
