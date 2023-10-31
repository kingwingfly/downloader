use super::error::{bmc_error, BmcResult};
use super::Model;
use crate::task::Task;

use snafu::OptionExt;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::oneshot::{self, Sender as OnceSender};
use uuid::Uuid;

type CreateResult = Result<Arc<Task>, crate::task::TaskError>;
type Message = Option<(String, OnceSender<CreateResult>)>;

pub struct TaskBmc {
    model: Model,
    tx: mpsc::Sender<Message>,
    jh: Option<std::thread::JoinHandle<()>>,
}

macro_rules! bmc_func {
    ($func: ident) => {
        pub fn $func(&self, id: Uuid) -> BmcResult<()> {
            let task = self
                .model
                .tasks
                .iter()
                .find(|t| t.id == id)
                .context(bmc_error::TaskNotFoundError { id })?
                .clone();
            task.$func()?;
            Ok(())
        }
    };
    ($($func: ident),+) => {
        $(bmc_func![$func];)+
    }
}

impl TaskBmc {
    pub fn new() -> Self {
        let (tx, mut rx) = mpsc::channel::<Message>(8);
        let jh = std::thread::spawn(move || {
            actix_rt::Runtime::new().unwrap().block_on(async move {
                let mut jhs = Vec::new();
                while let Some(Some((url, tx))) = rx.recv().await {
                    let task = Task::new(url);
                    match task {
                        Ok(task) => {
                            let task = Arc::new(task);
                            tx.send(Ok(task.clone())).ok();
                            let jh = actix_rt::spawn(async move { task.go().await });
                            jhs.push(jh);
                        }
                        Err(e) => {
                            tx.send(Err(e)).ok();
                        }
                    }
                }
                for jh in jhs {
                    jh.await.ok();
                }
                tracing::info!("all finished");
            });
        });
        Self {
            model: Model::new(),
            tx,
            jh: Some(jh),
        }
    }

    pub fn create<S>(&mut self, url: S) -> BmcResult<Uuid>
    where
        S: AsRef<str>,
    {
        let (tx, rx) = oneshot::channel::<CreateResult>();
        self.tx
            .blocking_send(Some((url.as_ref().to_string(), tx)))
            .unwrap();
        let new_task = rx.blocking_recv().unwrap()?;
        let uuid = new_task.id;
        self.model.tasks.push(new_task);
        Ok(uuid)
    }

    pub fn remove(&mut self, id: Uuid) -> BmcResult<()> {
        let i = self
            .model
            .tasks
            .iter()
            .position(|t| t.id == id)
            .context(bmc_error::TaskNotFoundError { id })?;
        self.model.tasks[i].cancel()?;
        self.model.tasks.swap_remove(i);
        Ok(())
    }

    pub fn process(&self) -> BmcResult<HashMap<String, (usize, usize)>> {
        let mut hp = HashMap::new();
        for t in self.model.tasks.iter() {
            let (filename, finished, total) = t.process_query().unwrap();
            println!("{}: {}/ {}", filename, finished, total);
            hp.insert(filename, (finished, total));
        }
        Ok(hp)
    }

    bmc_func![cancel, pause, continue_, revive, restart];
}

impl Drop for TaskBmc {
    fn drop(&mut self) {
        self.model.tasks.iter().for_each(|t| {
            t.cancel().ok();
        });
        self.tx.blocking_send(None).ok();
        self.jh.take().unwrap().join().ok();
        println!("task bmc droped");
    }
}

// endregion handler

#[cfg(test)]
mod tests {
    use super::*;
    use tracing_test::traced_test;

    #[test]
    fn create_test() {
        let mut task_bmc = TaskBmc::new();
        assert!(task_bmc
            .create("https://www.bilibili.com/video/BV1NN411F7HE")
            .is_ok());
        assert!(task_bmc.model.tasks.len() == 1);
        assert!(task_bmc.create("should fail").is_err());
        assert!(task_bmc.model.tasks.len() == 1);
    }

    #[traced_test]
    #[test]
    fn bmc_test() {
        let mut task_bmc = TaskBmc::new();
        // let id1 = task_bmc
        //     .create("https://www.bilibili.com/video/BV1NN411F7HE")
        //     .unwrap();
        // assert!(task_bmc.cancel(id1).is_ok());
        let id2 = task_bmc
            .create("https://www.bilibili.com/video/BV1NN411F7HE")
            .unwrap();
        std::thread::sleep(std::time::Duration::from_secs(2));
        assert!(task_bmc.pause(id2).is_ok());
        std::thread::sleep(std::time::Duration::from_secs(2));
        assert!(task_bmc.continue_(id2).is_ok());
        // std::thread::sleep(std::time::Duration::from_secs(10));

        // assert!(task_bmc.revive(id1).is_ok());
        // assert!(task_bmc.restart(id1).is_ok());
    }
}
