mod bilibili;
mod error;
mod info;
mod parser;
mod pixiv;
mod task_actor;

use crate::{config::get_config, utils::TempDirHandler};
use actix::Addr;
pub use error::*;
pub use info::Info;
use snafu::OptionExt;
use std::sync::Arc;
use task_actor::{Cancel, Continue_, Pause, RunTask, SetFilename, TaskActor};
use tokio::sync::oneshot;
use url::Url;
use uuid::Uuid;

use self::task_actor::ProgressQuery;

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub enum TaskType {
    BiliBili,
    Unknown,
}

macro_rules! task_func {
    (($func: ident, $msg: ident)) => {
        fn $func(&self) -> TaskResult<()> {
            self.addr().do_send($msg);
            Ok(())
        }
    };
    ($(($func: ident, $msg: ident)),+) => {
        $(task_func![($func, $msg)];)+
    }
}

pub trait TaskExe {
    type Info: Info;

    // Return (filename, infos)
    // filename: the filename of the video
    // infos: the video and audio infos which impl the Info trait
    async fn get_child_tasks(&self) -> TaskResult<(String, Vec<Self::Info>)>;

    fn addr(&self) -> &Addr<TaskActor>;
    fn url(&self) -> &Url;
    fn id(&self) -> &Uuid;

    async fn go(&self) -> TaskResult<()> {
        let (filename, infos) = self.get_child_tasks().await?;
        self.save(filename, infos).await?;
        Ok(())
    }

    async fn save(&self, filename: impl AsRef<str>, infos: Vec<Self::Info>) -> TaskResult<()> {
        let temp_dir = Arc::new(TempDirHandler::new(filename.as_ref()).unwrap());
        self.addr()
            .send(SetFilename(filename.as_ref().to_string()))
            .await??;
        let referer = self.referer()?;
        let mut rxs = vec![];
        for info in infos.into_iter() {
            let (tx, rx) = tokio::sync::oneshot::channel();
            let run_task = RunTask::new(info.suffix(), info.url(), &referer, temp_dir.clone(), tx);
            self.addr().send(run_task).await??;
            rxs.push(rx);
        }
        for rx in rxs {
            rx.await.unwrap()?;
        }
        temp_dir.save();
        Ok(())
    }

    fn progress_query(&self) -> TaskResult<(String, usize, usize, String)> {
        let (tx, rx) = oneshot::channel();
        self.addr().do_send(ProgressQuery::new(tx));
        Ok(rx.blocking_recv().unwrap().unwrap())
    }

    fn referer(&self) -> TaskResult<String> {
        match self.task_type() {
            TaskType::BiliBili => Ok("https://www.bilibili.com/".to_string()),
            TaskType::Unknown => task_error::UnknownTaskType.fail()?,
        }
    }

    fn cookie(&self) -> TaskResult<String> {
        match self.task_type() {
            TaskType::BiliBili => get_config("bili_cookie").context(task_error::ConfigNotFound),
            TaskType::Unknown => task_error::UnknownTaskType.fail()?,
        }
    }

    fn user_agent(&self) -> TaskResult<String> {
        get_config("user-agent").context(task_error::ConfigNotFound)
    }

    fn task_type(&self) -> TaskType {
        match self.url().host_str() {
            Some("bilibili.com") | Some("www.bilibili.com") => TaskType::BiliBili,
            Some(_) | None => TaskType::Unknown,
        }
    }

    task_func![(cancel, Cancel), (pause, Pause), (continue_, Continue_)];
}

pub fn new_task<S: AsRef<str>>(url: S) -> TaskResult<impl TaskExe<Info = impl Info>> {
    let url = url.as_ref().parse::<Url>()?;
    match url.host_str() {
        Some("bilibili.com") | Some("www.bilibili.com") => bilibili::BiliTask::new(url),
        Some(_) | None => task_error::UnknownTaskType.fail()?,
    }
}
