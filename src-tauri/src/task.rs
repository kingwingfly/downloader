use snafu::ResultExt;
use url::Url;

use crate::config::APP_CONFIG;
use crate::error::{task_error, TaskError, TaskResult};
use tracing::{debug, info, instrument, Level};

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
enum TaskType {
    BiliBili,
    Unknown,
}

// Task
trait TaskTrait {
    fn new<S: AsRef<str>>(url: S) -> TaskResult<Task>;
    async fn get_text(&self) -> TaskResult<String>;
    async fn get_task_type(&self) -> TaskType;
    async fn get_target(&self) -> TaskResult<Vec<Task>> {
        match self.get_task_type().await {
            TaskType::BiliBili => {
                todo!()
            }
            TaskType::Unknown => Err(TaskError::UnknownTaskType),
        }
    }
}

#[cfg_attr(test, derive(Debug))]
struct Task {
    url: Url,
}

impl TaskTrait for Task {
    fn new<S>(url: S) -> TaskResult<Self>
    where
        S: AsRef<str>,
    {
        Ok(Self {
            url: Url::parse(url.as_ref()).context(task_error::ParseUrlError {
                url: url.as_ref().to_string(),
            })?,
        })
    }

    #[instrument(level=Level::DEBUG, skip(self))]
    async fn get_text(&self) -> TaskResult<String> {
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
            200 => Ok(resp.text().await?),
            _ => Err(TaskError::StatusError),
        }
    }

    async fn get_task_type(&self) -> TaskType {
        match self.url.host_str() {
            Some("bilibili.com") | Some("www.bilibili.com") => TaskType::BiliBili,
            Some(_) | None => TaskType::Unknown,
        }
    }
}

// Task

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn new_task_test() {
        // url parse test
        assert!(Task::new("http://localhost:3000").is_ok());
        assert!(Task::new("https://www.bilibili.com").is_ok());
        let url = "example.com";
        let ret = Task::new(url);
        assert!(ret.is_err(), "wrong url parsed incorrectly: {}", url);
        if let Err(e) = ret {
            assert_eq!(e.to_string(), format!("Could not parse url: {url}"));
        };
        // url parse test
    }

    #[tokio::test]
    async fn get_test() {
        crate::config::config_init();
        let task = Task::new("https://bilibili.com").unwrap();
        let jh = tokio::spawn(async move { task.get_text().await });
        assert!(jh.await.unwrap().is_ok());
    }

    #[tokio::test]
    async fn get_task_type_test() {
        crate::config::config_init();
        let task = Task::new("https://bilibili.com").unwrap();
        let jh = tokio::spawn(async move { task.get_task_type().await });
        assert_eq!(jh.await.unwrap(), TaskType::BiliBili);
        let task = Task::new("https://www.bilibili.com/video/BV1a84y127b4/?spm_id_from=333.1007.top_right_bar_window_history.content.click&vd_source=9ea481f2d8d2d522899fe515c8f472c8").unwrap();
        let jh = tokio::spawn(async move { task.get_task_type().await });
        assert_eq!(jh.await.unwrap(), TaskType::BiliBili);
    }
}
