use super::error::{parse_error, ParseResult};
use scraper::{Html, Selector};
use snafu::{OptionExt, ResultExt};
use url::Url;

use super::Task;

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
            .inner_html()
            .replace("&amp;", "&");
        let json = script.split_at(20).1;
        let info_json: serde_json::Value =
            serde_json::from_str(&json).context(parse_error::JsonParseError)?;

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

        ret.push(Task::new(&videos[0].url).unwrap());
        ret.push(Task::new(&audios[0].url).unwrap());

        if ret.is_empty() {
            parse_error::NoTargetFound.fail()?;
        }
        Ok(ret)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::TaskExe;

    #[tracing_test::traced_test]
    #[actix_rt::test]
    async fn parse_bili_test() {
        crate::config::config_init().unwrap();
        let task = Task::new("https://www.bilibili.com/video/BV1NN411F7HE").unwrap();
        let html = task.get_html().await.unwrap();
        let ret = Parser::html(html).bilibili().unwrap();
        for t in ret {
            tracing::debug!("{}", t.url.to_string());
            assert!(!t.url.to_string().contains("&amp"));
        }
    }
}
