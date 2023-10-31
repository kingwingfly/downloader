mod error;
mod parser;
mod task_actor;

pub use error::TaskError;
use tokio::sync::oneshot;

use std::sync::Arc;

use crate::config::get_config;
use crate::task::parser::Parser;
use crate::utils::TempDirHandler;
use actix::{Actor, Addr};
use error::{task_error, TaskResult};
use parser::Info;
use scraper::Html;
use snafu::{OptionExt, ResultExt};
use task_actor::{Cancel, Continue_, Pause, Restart, Revive, RunTask, TaskActor};
use url::Url;
use uuid::Uuid;

#[cfg(test)]
use tracing::{instrument, Level};

use self::{
    task_actor::{ProcessQuery, SetFilename},
};

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub enum TaskType {
    BiliBili,
    Unknown,
}

// region Task

#[cfg_attr(test, derive(Debug))]
pub struct Task {
    pub id: Uuid,
    pub url: Url,
    addr: Addr<TaskActor>,
}

impl Task {
    pub fn new<S>(url: S) -> TaskResult<Self>
    where
        S: AsRef<str>,
    {
        Ok(Task {
            id: Uuid::new_v4(),
            url: Url::parse(url.as_ref())?,
            addr: TaskActor::new().start(),
        })
    }

    pub async fn go(&self) -> TaskResult<()> {
        let (filename, infos) = self.get_child_tasks().await?;
        self.save(filename, infos.into_iter().map(|i| i.url).collect())
            .await?;
        Ok(())
    }

    /// The filename should not contain the suffix
    /// `urls[0]` is the video urls
    /// `urls[1]` is the audio url
    /// Due to using pop to gain the url, so `&mut` needed
    #[cfg_attr(test, instrument(level=Level::DEBUG, skip(self, urls), fields(filename=filename.as_ref(), uuid=format!("<{:.5}...>", self.id.to_string())), err))]
    async fn save<S: AsRef<str>>(&self, filename: S, urls: Vec<Url>) -> TaskResult<()> {
        let temp_dir = Arc::new(TempDirHandler::new(filename.as_ref()).unwrap());
        self.addr
            .send(SetFilename(filename.as_ref().to_string()))
            .await??;
        for (i, url) in urls.into_iter().enumerate() {
            let (tx, rx) = tokio::sync::oneshot::channel();
            let run_task = RunTask::new(format!("{i}.mp4"), url, temp_dir.clone(), tx);
            self.addr.send(run_task).await??;
            rx.await.unwrap()?;
        }
        temp_dir.save();
        Ok(())
    }

    #[cfg_attr(test, instrument(level=Level::DEBUG, skip(self), fields(uuid=format!("<{:.5}...>", self.id.to_string()), rs), err))]
    async fn get_html(&self) -> TaskResult<Html> {
        crate::config::config_init().unwrap();
        // region get_resp
        let user_agent = get_config("user-agent").context(task_error::ConfigNotFound)?;
        let cookie = self.get_cookie()?;
        let client = reqwest::Client::builder()
            .user_agent(user_agent)
            .build()
            .unwrap();
        let resp = client
            .get(self.url.as_str())
            .header(reqwest::header::COOKIE, cookie)
            .send()
            .await?;
        // endregion get_resp
        let status = resp.status().as_u16();
        #[cfg(test)]
        tracing::Span::current().record("rs", status);
        match status {
            200 => Ok(Html::parse_document(&resp.text().await?)),
            _ => task_error::StatusError.fail()?,
        }
    }

    #[cfg_attr(test, instrument(level=Level::DEBUG, skip(self), fields(uuid=format!("<{:.5}...>", self.id.to_string())), err))]
    async fn get_child_tasks(&self) -> TaskResult<(String, Vec<Info>)> {
        match self.get_task_type() {
            TaskType::BiliBili => {
                let html = self.get_html().await?;
                Ok(Parser::html(html).bilibili()?)
            }
            TaskType::Unknown => task_error::UnknownTaskType.fail()?,
        }
    }

    #[cfg_attr(test, instrument(level=Level::DEBUG, skip(self), fields(uuid=format!("<{:.5}...>", self.id.to_string())), err))]
    fn get_cookie(&self) -> TaskResult<String> {
        match self.get_task_type() {
            TaskType::BiliBili => get_config("bili_cookie").context(task_error::ConfigNotFound),
            TaskType::Unknown => task_error::UnknownTaskType.fail()?,
        }
    }

    #[cfg_attr(test, instrument(level=Level::DEBUG, skip(self), fields(url=self.url.as_str()), ret))]
    fn get_task_type(&self) -> TaskType {
        match self.url.host_str() {
            Some("bilibili.com") | Some("www.bilibili.com") => TaskType::BiliBili,
            Some(_) | None => TaskType::Unknown,
        }
    }

    pub fn process_query(&self) -> TaskResult<(String, usize, usize)> {
        let (tx, rx) = oneshot::channel();
        self.addr.do_send(ProcessQuery::new(tx));
        Ok(rx.blocking_recv().unwrap().unwrap())
    }
}

macro_rules! task_func {
    (($func: ident, $msg: ident)) => {
        #[cfg_attr(test, instrument(level=Level::DEBUG, skip(self), fields(uuid=format!("<{:.5}...>", self.id.to_string())), err))]
        pub fn $func(&self) -> TaskResult<()> {
            self.addr.do_send($msg);
            Ok(())
        }
    };
    ($(($func: ident, $msg: ident)),+) => {
        $(task_func![($func, $msg)];)+
    }
}

impl Task {
    task_func![
        (cancel, Cancel),
        (pause, Pause),
        (continue_, Continue_),
        (revive, Revive),
        (restart, Restart)
    ];
}

// endregion Task

#[cfg(test)]
mod tests {

    use super::*;

    #[actix_rt::test]
    async fn new_task_test() {
        // region url parse test
        assert!(Task::new("http://localhost:3000").is_ok());
        assert!(Task::new("https://www.bilibili.com").is_ok());
        let url = "example.com";
        let ret = Task::new(url);
        assert!(ret.is_err(), "wrong url parsed incorrectly: {}", url);
        if let Err(e) = ret {
            assert_eq!(e.to_string(), format!("Could not parse url: {url}"));
        };
        // endregion url parse test
    }

    #[actix_rt::test]
    async fn get_test() {
        assert!(crate::config::config_init().is_ok());
        let task = Task::new("https://bilibili.com").unwrap();
        assert!(task.get_html().await.is_ok());
    }

    #[actix_rt::test]
    async fn get_task_type_test() {
        assert!(crate::config::config_init().is_ok());
        let task = Task::new("https://bilibili.com").unwrap();
        assert_eq!(task.get_task_type(), TaskType::BiliBili);
        let task = Task::new("https://www.bilibili.com/video/BV1NN411F7HE").unwrap();
        assert_eq!(task.get_task_type(), TaskType::BiliBili);
    }

    #[tracing_test::traced_test]
    #[actix_rt::test]
    async fn task_go_test() {
        assert!(crate::config::config_init().is_ok());
        let task = Arc::new(Task::new("https://www.bilibili.com/video/BV1NN411F7HE").unwrap());
        let task_c = task.clone();
        let jh = actix_rt::spawn(async move { task_c.go().await });
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        assert!(task.pause().is_ok());
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        assert!(task.continue_().is_ok());
        assert!(jh.await.is_ok());
    }
}
