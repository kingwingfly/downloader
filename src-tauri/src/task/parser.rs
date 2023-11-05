use super::error::{parse_error, ParseResult};

use scraper::{Html, Selector};
use snafu::{OptionExt, ResultExt};
use url::Url;

pub trait Info {
    fn suffix(&self) -> String;
    fn url(&self) -> Url;
}

pub struct Parser {
    html: Html,
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

impl Parser {
    pub fn html(html: Html) -> Self {
        Self { html }
    }

    pub fn bilibili(&self) -> ParseResult<(String, Vec<BiliInfo>)> {
        let mut ret = vec![];
        let selector = Selector::parse("head script").unwrap();
        let json = self
            .html
            .select(&selector)
            .nth(3)
            .context(parse_error::BiliPlayInfoNotFound)?
            .inner_html()
            .replace("&amp;", "&");
        let json = json.split_at(20).1;
        let info_json: serde_json::Value =
            serde_json::from_str(json).context(parse_error::JsonParseError)?;

        let selector = Selector::parse("h1.video-title").unwrap();
        let filename = self
            .html
            .select(&selector)
            .nth(0)
            .context(parse_error::BiliPlayInfoNotFound)?
            .inner_html();

        let parse = |ty: &str| -> ParseResult<Vec<BiliInfo>> {
            Ok(info_json
                .pointer(&format!("/data/dash/{ty}"))
                .context(parse_error::BiliPlayInfoNotFound)?
                .as_array()
                .context(parse_error::BiliPlayInfoNotFound)?
                .iter()
                .rev()
                .filter_map(|v| serde_json::from_value(v.clone()).ok())
                .collect())
        };

        let mut videos = parse("video")?;

        let mut audios = parse("audio")?;

        // Maybe Hi-res exists
        if let Some(a) = info_json.pointer("/data/flac/audio") {
            audios.push(serde_json::from_value(a.clone()).unwrap());
        }

        ret.push(videos.pop().unwrap());
        ret.push(audios.pop().unwrap());

        if ret.is_empty() {
            parse_error::NoTargetFound.fail()?;
        }

        Ok((filename, ret))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::Task;

    #[tracing_test::traced_test]
    #[actix_rt::test]
    async fn parse_bili_test() {
        let task = Task::new("https://www.bilibili.com/video/BV1Z84y1D7DJ").unwrap();
        let html = task.get_html().await.unwrap();
        let ret = Parser::html(html).bilibili().unwrap();
        for info in ret.1 {
            assert!(!info.url.to_string().contains("&amp"));
        }
    }

    #[test]
    fn suffix_test() {
        assert_eq!(mime_suffix("video/mp4"), "mp4");
        assert_eq!(mime_suffix("audio/mp4"), "m4a");
    }
}
