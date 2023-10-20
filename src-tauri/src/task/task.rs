use snafu::ResultExt;
use url::Url;

use super::error::{task_error, TaskError, TaskResult};
use crate::config::APP_CONFIG;
use scraper::Html;
use tracing::{debug, info, instrument, Level};
use uuid::Uuid;

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub enum TaskType {
    BiliBili,
    Unknown,
}

// Task
pub trait TaskExe {
    fn new<S: AsRef<str>>(url: S) -> TaskResult<Task>;
    fn go(&self) -> TaskResult<()>;
    async fn get_html(&self) -> TaskResult<Html>;
    fn get_task_type(&self) -> TaskType;
    fn cancel(&self) -> TaskResult<()>;
    fn pause(&self) -> TaskResult<()>;
    fn continue_(&self) -> TaskResult<()>;
    fn revive(&self) -> TaskResult<()>;
    fn restart(&self) -> TaskResult<()>;
    async fn get_target(&self) -> TaskResult<Vec<Task>> {
        match self.get_task_type() {
            TaskType::BiliBili => {
                let html = self.get_html().await?;
                // let target = parser.html(html).bilibili()?;
                todo!()
            }
            TaskType::Unknown => Err(TaskError::UnknownTaskType),
        }
    }
}

#[cfg_attr(test, derive(Debug))]
pub struct Task {
    pub id: Uuid,
    url: Url,
}

impl TaskExe for Task {
    fn new<S>(url: S) -> TaskResult<Self>
    where
        S: AsRef<str>,
    {
        // Todo maybe use channel to push task in another thread???
        Ok(Self {
            id: Uuid::new_v4(),
            url: Url::parse(url.as_ref()).context(task_error::ParseUrlError {
                url: url.as_ref().to_string(),
            })?,
        })
    }
    fn go(&self) -> TaskResult<()> {
        todo!()
    }

    #[cfg_attr(test, instrument(level=Level::DEBUG, skip(self), fields(uuid=format!("<{:.5}...>", self.id.to_string())), err))]
    async fn get_html(&self) -> TaskResult<Html> {
        // region get_resp
        let user_agent = APP_CONFIG.get().unwrap().get_string("user-agent").unwrap();
        let cookie = APP_CONFIG.get().unwrap().get_string("cookie").unwrap();
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
        info!("Response status: {}", resp.status());
        match resp.status().as_u16() {
            200 => Ok(Html::parse_document(&resp.text().await?)),
            _ => Err(TaskError::StatusError),
        }
    }

    #[instrument(level=Level::DEBUG, skip(self))]
    fn get_task_type(&self) -> TaskType {
        match self.url.host_str() {
            Some("bilibili.com") | Some("www.bilibili.com") => TaskType::BiliBili,
            Some(_) | None => TaskType::Unknown,
        }
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

// Task

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
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

    #[tokio::test]
    async fn get_test() {
        crate::config::config_init();
        let task = Task::new("https://bilibili.com").unwrap();
        assert!(task.get_html().await.is_ok());
    }

    #[test]
    fn get_task_type_test() {
        crate::config::config_init();
        let task = Task::new("https://bilibili.com").unwrap();
        assert_eq!(task.get_task_type(), TaskType::BiliBili);
        let task = Task::new("https://www.bilibili.com/video/BV1a84y127b4/?spm_id_from=333.1007.top_right_bar_window_history.content.click&vd_source=9ea481f2d8d2d522899fe515c8f472c8").unwrap();
        assert_eq!(task.get_task_type(), TaskType::BiliBili);
    }
}
