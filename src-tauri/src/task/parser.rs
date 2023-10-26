use super::error::{parse_error, ParseResult};
use scraper::{Html, Selector};
use snafu::{OptionExt, ResultExt};
use url::Url;

pub struct Parser {
    html: Html,
}

#[cfg_attr(test, derive(Debug))]
#[derive(serde::Deserialize)]
pub struct Info {
    #[serde(rename(deserialize = "base_url"))]
    pub url: Url,
    #[serde(default)]
    pub filename: String,
    width: usize,
    height: usize,
}

impl Parser {
    pub fn html(html: Html) -> Self {
        Self { html }
    }

    pub fn bilibili(&self) -> ParseResult<Vec<Info>> {
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
            serde_json::from_str(&json).context(parse_error::JsonParseError)?;

        let selector = Selector::parse("h1.video-title").unwrap();
        let filename = self
            .html
            .select(&selector)
            .nth(0)
            .context(parse_error::BiliPlayInfoNotFound)?
            .inner_html();

        let parse = |ty: &str| -> ParseResult<Vec<Info>> {
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

        ret.push(videos.pop().unwrap());
        ret.push(audios.pop().unwrap());

        if ret.is_empty() {
            parse_error::NoTargetFound.fail()?;
        }
        ret[0].filename = filename;
        Ok(ret)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::{Task, TaskExe};

    #[tracing_test::traced_test]
    #[actix_rt::test]
    async fn parse_bili_test() {
        crate::config::config_init().unwrap();
        let task = Task::new("https://www.bilibili.com/video/BV1NN411F7HE").unwrap();
        let html = task.get_html().await.unwrap();
        let ret = Parser::html(html).bilibili().unwrap();
        for t in ret {
            tracing::debug!("{}", t.url.to_string());
            tracing::debug!("{}", t.filename.to_string());
            assert!(!t.url.to_string().contains("&amp"));
        }
    }
}
