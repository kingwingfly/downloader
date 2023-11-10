use super::{
    error::{parse_error, ParseResult},
    info::Info,
};

use snafu::OptionExt;

pub struct JsonParser {
    json: serde_json::Value,
}

impl JsonParser {
    pub fn new(json: serde_json::Value) -> Self {
        Self { json }
    }

    pub fn get_string(&self, pointer: &str) -> ParseResult<String> {
        Ok(self
            .json
            .pointer(pointer)
            .context(parse_error::InfoNotFound)?
            .to_string())
    }

    #[allow(unused)]
    pub fn get_info<T>(&self, pointer: &str) -> ParseResult<Box<dyn Info>>
    where
        for<'de> T: serde::Deserialize<'de> + Info + 'static,
    {
        Ok(Box::new(serde_json::from_value::<T>(
            self.json
                .pointer(pointer)
                .context(parse_error::InfoNotFound)?
                .clone(),
        )?) as Box<dyn Info>)
    }

    pub fn get_info_array<T>(&self, pointer: &str) -> ParseResult<Vec<Box<dyn Info>>>
    where
        for<'de> T: serde::Deserialize<'de> + Info + 'static,
    {
        Ok(self
            .json
            .pointer(pointer)
            .context(parse_error::InfoNotFound)?
            .as_array()
            .context(parse_error::InfoNotFound)?
            .iter()
            .rev()
            .filter_map(|v| serde_json::from_value::<T>(v.clone()).ok())
            .map(|i| Box::new(i) as Box<dyn Info>)
            .collect())
    }
}
