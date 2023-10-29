mod error;
mod parser;
mod task_actor;

pub use error::TaskError;

use std::sync::Arc;

use crate::config::get_config;
use crate::task::parser::Parser;
use crate::utils::TempDirHandler;
use actix::{Actor, Addr};
use error::{task_error, TaskResult};
use parser::Info;
use scraper::Html;
use snafu::{OptionExt, ResultExt};
use task_actor::{Cancel, Continue_, Pause, RunTask, TaskActor};
use url::Url;
use uuid::Uuid;

#[cfg(test)]
use tracing::{instrument, Level};

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub enum TaskType {
    BiliBili,
    Unknown,
}

// region Task
pub trait TaskExe {
    async fn go(&self) -> TaskResult<()>;
    async fn get_html(&self) -> TaskResult<Html>;
    async fn save<S: AsRef<str>>(&self, filename: S, infos: Vec<Url>) -> TaskResult<()>;
    fn cancel(&self) -> TaskResult<()>;
    fn pause(&self) -> TaskResult<()>;
    fn continue_(&self) -> TaskResult<()>;
    fn revive(&self) -> TaskResult<()>;
    fn restart(&self) -> TaskResult<()>;
}

#[cfg_attr(test, derive(Debug))]
pub struct Task {
    pub id: Uuid,
    pub url: Url,
    addr: Addr<TaskActor>,
}

impl Task {
    pub fn new<S>(url: S) -> TaskResult<Arc<Self>>
    where
        S: AsRef<str>,
    {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let url = url.as_ref().to_string();
        std::thread::spawn(move || {
            actix_rt::Runtime::new().unwrap().block_on(async {
                let task = Arc::new(Task {
                    id: Uuid::new_v4(),
                    url: Url::parse(&url).unwrap(),
                    addr: TaskActor::new().start(),
                });
                tx.send(task.clone()).ok();
                task.go().await.unwrap();
            });
        });
        let task = rx.blocking_recv().unwrap();
        Ok(task)
    }

    #[cfg_attr(test, instrument(level=Level::DEBUG, skip(self), fields(url=self.url.as_str()), ret))]
    fn get_task_type(&self) -> TaskType {
        match self.url.host_str() {
            Some("bilibili.com") | Some("www.bilibili.com") => TaskType::BiliBili,
            Some(_) | None => TaskType::Unknown,
        }
    }

    #[cfg_attr(test, instrument(level=Level::DEBUG, skip(self), fields(uuid=format!("<{:.5}...>", self.id.to_string())), err))]
    async fn get_child_tasks(&self) -> TaskResult<Vec<Info>> {
        match self.get_task_type() {
            TaskType::BiliBili => {
                let html = self.get_html().await?;
                let child_tasks = Parser::html(html).bilibili()?;
                debug_assert!(!child_tasks.is_empty());
                Ok(child_tasks)
            }
            TaskType::Unknown => task_error::UnknownTaskType.fail()?,
        }
    }
}

impl TaskExe for Task {
    async fn go(&self) -> TaskResult<()> {
        let infos = self.get_child_tasks().await?;
        self.save(
            infos[0].filename.clone(),
            infos.into_iter().map(|i| i.url).collect(),
        )
        .await?;
        Ok(())
    }

    #[cfg_attr(test, instrument(level=Level::DEBUG, skip(self), fields(uuid=format!("<{:.5}...>", self.id.to_string()), rs), err))]
    async fn get_html(&self) -> TaskResult<Html> {
        crate::config::config_init().unwrap();
        // region get_resp
        let user_agent = get_config("user-agent").context(task_error::ConfigNotFound)?;
        let cookie = get_config("cookie").context(task_error::ConfigNotFound)?;
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

    /// The filename should not contain the suffix
    /// `urls[0]` is the video url
    /// `urls[1]` is the audio url
    /// Due to using pop to gain the url, so `&mut` needed
    #[cfg_attr(test, instrument(level=Level::DEBUG, skip(self, urls), fields(filename=filename.as_ref(), uuid=format!("<{:.5}...>", self.id.to_string())), err))]
    async fn save<S: AsRef<str>>(&self, filename: S, mut urls: Vec<Url>) -> TaskResult<()> {
        let temp_dir = Arc::new(TempDirHandler::new(filename).unwrap());
        let (tx, rx) = tokio::sync::oneshot::channel();
        let child_task = RunTask::new("aac", urls.pop().unwrap(), temp_dir.clone(), tx);
        self.addr.send(child_task).await??;
        rx.await.unwrap()?;

        let (tx, rx) = tokio::sync::oneshot::channel();
        let child_task = RunTask::new("mp4", urls.pop().unwrap(), temp_dir.clone(), tx);
        self.addr.send(child_task).await??;
        rx.await.unwrap()?;
        Ok(())
    }

    #[cfg_attr(test, instrument(level=Level::DEBUG, skip(self), fields(uuid=format!("<{:.5}...>", self.id.to_string())), err))]
    fn cancel(&self) -> TaskResult<()> {
        self.addr.do_send(Cancel);
        Ok(())
    }

    #[cfg_attr(test, instrument(level=Level::DEBUG, skip(self), fields(uuid=format!("<{:.5}...>", self.id.to_string())), err))]
    fn pause(&self) -> TaskResult<()> {
        self.addr.do_send(Pause);
        Ok(())
    }

    #[cfg_attr(test, instrument(level=Level::DEBUG, skip(self), fields(uuid=format!("<{:.5}...>", self.id.to_string())), err))]
    fn continue_(&self) -> TaskResult<()> {
        self.addr.do_send(Continue_);
        Ok(())
    }

    #[cfg_attr(test, instrument(level=Level::DEBUG, skip(self), fields(uuid=format!("<{:.5}...>", self.id.to_string())), err))]
    fn revive(&self) -> TaskResult<()> {
        Ok(())
    }

    #[cfg_attr(test, instrument(level=Level::DEBUG, skip(self), fields(uuid=format!("<{:.5}...>", self.id.to_string())), err))]
    fn restart(&self) -> TaskResult<()> {
        Ok(())
    }
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
    #[test]
    fn task_go_test() {
        assert!(crate::config::config_init().is_ok());
        let task = Task::new("https://www.bilibili.com/video/BV1NN411F7HE").unwrap();
        std::thread::sleep(std::time::Duration::from_secs(1));
        assert!(task.pause().is_ok());
        std::thread::sleep(std::time::Duration::from_secs(5));
        assert!(task.continue_().is_ok());
        // std::thread::sleep(std::time::Duration::from_secs(10));
    }
}
