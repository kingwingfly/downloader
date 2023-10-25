use super::error::{task_error, TaskResult};
use super::task_actor::{RunTask, TaskActor};
use crate::config::get_config;
use crate::task::parser::Parser;
use actix::{Actor, Addr};
use scraper::Html;
use snafu::{OptionExt, ResultExt};
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
    async fn save(&self) -> TaskResult<()>;
    fn cancel(&self) -> TaskResult<()>;
    fn pause(&self) -> TaskResult<()>;
    fn continue_(&self) -> TaskResult<()>;
    fn revive(&self) -> TaskResult<()>;
    fn restart(&self) -> TaskResult<()>;
}

#[cfg_attr(test, derive(Debug))]
pub struct Task {
    pub id: Uuid,
    pub(super) url: Url,
    addr: Addr<TaskActor>,
}

impl Task {
    pub fn new<S>(url: S) -> TaskResult<Self>
    where
        S: AsRef<str>,
    {
        // Todo maybe use channel to push task in another thread???
        let addr = TaskActor::new().start();
        Ok(Self {
            id: Uuid::new_v4(),
            url: Url::parse(url.as_ref()).context(task_error::ParseUrlError {
                url: url.as_ref().to_string(),
            })?,
            addr,
        })
    }

    #[cfg_attr(test, instrument(level=Level::DEBUG, skip(self), fields(url=self.url.as_str()), ret))]
    fn get_task_type(&self) -> TaskType {
        match self.url.host_str() {
            Some("bilibili.com") | Some("www.bilibili.com") => TaskType::BiliBili,
            Some(_) | None => TaskType::Unknown,
        }
    }

    // #[cfg_attr(test, instrument(level=Level::DEBUG, skip(self), fields(uuid=format!("<{:.5}...>", self.id.to_string())), err))]
    async fn get_child_tasks(&self) -> TaskResult<Vec<Task>> {
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
        for t in self.get_child_tasks().await? {
            t.save().await?;
        }
        Ok(())
    }

    #[cfg_attr(test, instrument(level=Level::DEBUG, skip(self), fields(uuid=format!("<{:.5}...>", self.id.to_string()), rs), err))]
    async fn get_html(&self) -> TaskResult<Html> {
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
        tracing::Span::current().record("rs", &status);
        match status {
            200 => Ok(Html::parse_document(&resp.text().await?)),
            _ => task_error::StatusError.fail()?,
        }
    }

    #[cfg_attr(test, instrument(level=Level::DEBUG, skip(self), fields(uuid=format!("<{:.5}...>", self.id.to_string())), err))]
    async fn save(&self) -> TaskResult<()> {
        let child_task = RunTask::new(self.url.clone());
        self.addr.send(child_task).await??;
        Ok(())
    }

    #[cfg_attr(test, instrument(level=Level::DEBUG, skip(self), fields(uuid=format!("<{:.5}...>", self.id.to_string())), err))]
    fn cancel(&self) -> TaskResult<()> {
        Ok(())
    }

    #[cfg_attr(test, instrument(level=Level::DEBUG, skip(self), fields(uuid=format!("<{:.5}...>", self.id.to_string())), err))]
    fn pause(&self) -> TaskResult<()> {
        Ok(())
    }

    #[cfg_attr(test, instrument(level=Level::DEBUG, skip(self), fields(uuid=format!("<{:.5}...>", self.id.to_string())), err))]
    fn continue_(&self) -> TaskResult<()> {
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
    #[actix_rt::test]
    async fn task_go_test() {
        assert!(crate::config::config_init().is_ok());
        let task = Task::new("https://www.bilibili.com/video/BV1NN411F7HE").unwrap();
        assert!(task.go().await.is_ok());
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
}
