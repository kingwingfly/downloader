use super::error::{parse_error, ParseResult};
use scraper::{Html, Selector};
use snafu::{OptionExt, ResultExt};
use url::Url;

use super::{Task, TaskExe};

pub struct Parser {
    html: Html,
}

#[cfg_attr(test, derive(Debug))]
#[derive(serde::Deserialize)]
struct Info {
    #[serde(rename(deserialize = "base_url"))]
    url: Url,
    width: usize,
    height: usize,
}

impl Parser {
    pub fn html(html: Html) -> Self {
        Self { html }
    }

    pub fn bilibili(&self) -> ParseResult<Vec<Task>> {
        let mut ret = vec![];
        let selector = Selector::parse("head script").unwrap();
        let script = self
            .html
            .select(&selector)
            .nth(3)
            .context(parse_error::BiliPlayInfoNotFound)?
            .inner_html();
        let info = script.split_at(20).1;
        let info_json: serde_json::Value =
            serde_json::from_str(info).context(parse_error::JsonParseError)?;

        let parse = |ty: &str| -> ParseResult<Vec<Info>> {
            Ok(info_json
                .pointer(&format!("/data/dash/{ty}"))
                .context(parse_error::BiliPlayInfoNotFound)?
                .as_array()
                .context(parse_error::BiliPlayInfoNotFound)?
                .iter()
                .filter_map(|v| serde_json::from_value(v.clone()).ok())
                .collect())
        };

        let videos = parse("video")?;
        let audios = parse("audio")?;

        #[cfg(test)]
        {
            tracing::debug!("{:#?}", videos);
            tracing::debug!("{:#?}", audios);
        }

        if ret.is_empty() {
            parse_error::NoTargetFound.fail()?;
        }
        Ok(ret)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tracing_test::traced_test]
    #[tokio::test]
    async fn parse_bili_test() {
        crate::config::config_init();
        let task = Task::new("https://www.bilibili.com/video/BV1NN411F7HE").unwrap();
        let ret = task.go().await;
    }
}
