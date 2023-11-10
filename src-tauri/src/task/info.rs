use url::Url;

pub trait Info: std::fmt::Debug {
    fn suffix(&self) -> String;
    fn url(&self) -> Url;
}

#[derive(serde::Deserialize, Debug)]
pub struct BiliInfo {
    #[serde(rename(deserialize = "base_url"))]
    pub url: Url,
    pub width: usize,
    pub height: usize,
    #[serde(deserialize_with = "from_mime", rename(deserialize = "mime_type"))]
    pub suffix: String,
}

impl Info for BiliInfo {
    fn suffix(&self) -> String {
        self.suffix.to_owned()
    }

    fn url(&self) -> Url {
        self.url.to_owned()
    }
}

fn mime_suffix<S: AsRef<str>>(mime_type: S) -> String {
    new_mime_guess::get_mime_extensions_str(mime_type.as_ref())
        .unwrap()
        .first()
        .unwrap()
        .to_string()
}

fn from_mime<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = serde::Deserialize::deserialize(deserializer)?;
    Ok(mime_suffix(s))
}
