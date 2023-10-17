use config::{Config, Map, Source, Value, ValueKind};
use std::{collections::HashSet, sync::OnceLock};

pub(crate) static APP_CONFIG: OnceLock<Config> = OnceLock::new();
pub(crate) const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.1 Safari/605.1.15";

pub fn config_init() {
    APP_CONFIG.get_or_init(|| {
        Config::builder()
            .set_default("user-agent", USER_AGENT)
            .unwrap()
            .set_default("cookie", "")
            .unwrap()
            .add_source(KeySource)
            .build()
            .unwrap()
    });
}

#[derive(Debug, Clone)]
struct KeySource;

impl Source for KeySource {
    fn clone_into_box(&self) -> Box<dyn Source + Send + Sync> {
        Box::new(self.to_owned())
    }
    fn collect(&self) -> Result<config::Map<String, config::Value>, config::ConfigError> {
        Ok(MapWrapper::from(HashSet::from([("cookie", "")])).0)
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

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn init_config_test() {
        config_init();
        assert_eq!(
            APP_CONFIG.get().unwrap().get_string("user-agent").unwrap(),
            USER_AGENT
        );
        assert_eq!(APP_CONFIG.get().unwrap().get_string("cookie").unwrap(), "")
    }
}
