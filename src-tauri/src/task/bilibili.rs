use actix::{Actor, Addr};
use snafu::OptionExt;
use url::Url;
use uuid::Uuid;

use crate::{config::get_config, task::parser::JsonParser};

use super::{error::TaskResult, info::BiliInfo, task_actor::TaskActor, task_error, TaskExe};

pub struct BiliTask {
    id: Uuid,
    url: Url,
    addr: Addr<TaskActor>,
}

impl BiliTask {
    pub fn new<S>(url: S) -> TaskResult<Self>
    where
        S: AsRef<str>,
    {
        Ok(Self {
            id: Uuid::new_v4(),
            url: Url::parse(url.as_ref())?,
            addr: TaskActor::new().start(),
        })
    }
}

impl TaskExe for BiliTask {
    type Info = BiliInfo;

    async fn get_child_tasks(&self) -> TaskResult<(String, Vec<Self::Info>)> {
        let client = reqwest::Client::new();

        let bvid = self
            .url()
            .path_segments()
            .context(task_error::BvidNotFound)?
            .nth(1)
            .context(task_error::BvidNotFound)?;

        let api = "https://api.bilibili.com/x/web-interface/view";
        let resp = client.get(api).query(&[("bvid", bvid)]).send().await?;

        let json = resp.json::<serde_json::Value>().await?;
        #[cfg(test)]
        std::fs::write(
            "../example/bili_info.json",
            serde_json::to_string_pretty(&json).unwrap(),
        )
        .unwrap();
        let parser = JsonParser::new(json);
        let title = parser.get_string("/data/title")?;
        let cid = parser.get_string("/data/cid")?;
        // dbg!(&title, &cid);
        let api = "https://api.bilibili.com/x/player/wbi/playurl";

        let resp = client
            .get(api)
            .query(&[
                ("bvid", bvid),
                ("cid", &cid),
                ("qn", "127"),
                ("fourk", "1"),
                ("fnval", "1040"), // 16 | 1024 = 1040
                ("fnver", "0"),
            ])
            .header("cookie", self.cookie()?)
            .header("user-agent", self.user_agent()?)
            .send()
            .await?;
        let json = resp.json::<serde_json::Value>().await?;
        #[cfg(test)]
        std::fs::write(
            "../example/bili_video_info.json",
            serde_json::to_string_pretty(&json).unwrap(),
        )
        .unwrap();
        let parser = JsonParser::new(json);
        let mut videos = parser.get_info_array::<BiliInfo>("/data/dash/video")?;
        let mut audios = parser.get_info_array::<BiliInfo>("/data/dash/audio")?;
        let infos = vec![videos.pop().unwrap(), audios.pop().unwrap()];
        Ok((title, infos))
    }

    fn cookie(&self) -> TaskResult<String> {
        get_config("bili_cookie").context(task_error::ConfigNotFound)
    }

    fn addr(&self) -> &actix::Addr<super::task_actor::TaskActor> {
        &self.addr
    }

    fn url(&self) -> &url::Url {
        &self.url
    }

    fn id(&self) -> &Uuid {
        &self.id
    }
}

#[cfg(test)]
mod bilibili_test {
    use super::*;

    #[actix_rt::test]
    async fn bili_child_task_test() {
        let task = BiliTask::new("https://www.bilibili.com/video/BV1EC4y1V7ho").unwrap();
        let (filename, infos) = task.get_child_tasks().await.unwrap();
        println!("{}", filename);
        println!("{:#?}", infos);
    }

    #[actix_rt::test]
    async fn test_bilibili() {
        let task = BiliTask::new("https://www.bilibili.com/video/BV1EC4y1V7ho").unwrap();
        task.go().await.unwrap();
    }
}
