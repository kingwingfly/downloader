use super::error::{parse_error, ParseResult};
use scraper::{Html, Selector};
use snafu::OptionExt;

use super::{Task, TaskExe};

pub struct Parser {
    html: Html,
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
        let info_json: serde_json::Value = serde_json::from_str(info).unwrap();

        // This closure helps extracting video and audio from info_json
        let get_urls = |ty: &str| -> ParseResult<Vec<String>> {
            Ok(info_json
                .pointer(&format!("/data/dash/{ty}"))
                .context(parse_error::JsonParseError)?
                .as_array()
                .context(parse_error::JsonParseError)?
                .iter()
                .filter_map(|v| Some(v.pointer("/base_url")?.to_string()))
                .collect())
        };

        let v_urls = get_urls("video")?;
        let a_urls = get_urls("audio")?;

        tracing::debug!("{:#?}", v_urls);
        tracing::debug!("{:#?}", a_urls);

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
