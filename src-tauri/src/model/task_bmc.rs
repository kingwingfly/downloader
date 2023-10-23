use super::error::{bmc_error, BmcResult};
use super::Model;
use crate::task::{Task, TaskExe};
use actix::prelude::*;
use snafu::OptionExt;
use std::rc::Rc;
use uuid::Uuid;

#[cfg(test)]
use tracing::debug;

pub struct TaskBmc {
    model: Model,
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
        Self {
            model: Model::new(),
        }
    }

    pub fn create<S>(&mut self, url: S) -> BmcResult<Uuid>
    where
        S: AsRef<str>,
    {
        let new_task = Task::new(url)?;
        let uuid = new_task.id;
        self.model.tasks.push(Rc::new(new_task));
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

    bmc_func![cancel, pause, continue_, revive, restart];
}

impl Actor for TaskBmc {
    type Context = Context<Self>;
}

#[cfg(test)]
#[derive(Message)]
#[rtype(result = "BmcResult<()>")]
struct Ping;

#[cfg(test)]
impl Handler<Ping> for TaskBmc {
    type Result = BmcResult<()>;

    fn handle(&mut self, _msg: Ping, _ctx: &mut Self::Context) -> Self::Result {
        Ok(())
    }
}

// region handler

#[derive(Message)]
#[rtype(result = "BmcResult<Uuid>")]
struct Create<S: AsRef<str>>(S);

impl<S> Handler<Create<S>> for TaskBmc
where
    S: AsRef<str>,
{
    type Result = BmcResult<Uuid>;

    fn handle(&mut self, msg: Create<S>, _ctx: &mut Self::Context) -> Self::Result {
        self.create(msg.0)
    }
}

macro_rules! handler_gen {
    ($msg_name:ident) => {
        #[derive(Message)]
        #[rtype(result = "BmcResult<()>")]
        struct $msg_name(Uuid);

        impl Handler<$msg_name> for TaskBmc {
            type Result = BmcResult<()>;

            fn handle(&mut self, msg: $msg_name, _ctx: &mut Self::Context) -> Self::Result {
                casey::lower![self.$msg_name(msg.0)]
            }
        }
    };
    ($($msg_name:ident),+) => {
        $(handler_gen![$msg_name];)+
    }
}

handler_gen![Cancel, Pause, Continue_, Revive, Restart, Remove];

// endregion handler

#[cfg(test)]
mod tests {
    use super::*;
    use tracing_test::traced_test;

    #[tokio::test]
    async fn create_test() {
        let mut task_bmc = TaskBmc::new();
        assert!(task_bmc.create("http://bilibili.com").is_ok());
        assert!(task_bmc.model.tasks.len() == 1);
        assert!(task_bmc.create("should fail").is_err());
        assert!(task_bmc.model.tasks.len() == 1);
    }

    #[traced_test]
    #[test]
    fn handle_test() {
        let mut task_bmc = TaskBmc::new();
        let id1 = task_bmc.create("http://bilibili.com").unwrap();
        assert!(task_bmc.cancel(id1).is_ok());
        let id2 = task_bmc.create("http://bilibili.com").unwrap();
        assert!(task_bmc.pause(id2).is_ok());
        assert!(task_bmc.continue_(id2).is_ok());
        assert!(task_bmc.revive(id1).is_ok());
        assert!(task_bmc.restart(id1).is_ok());
    }

    #[traced_test]
    #[test]
    fn actix_test() {
        let system = System::new();
        let exe = async {
            let task_bmc = TaskBmc::new();
            let addr = task_bmc.start();
            let ping = Ping;
            assert!(addr.send(ping).await.is_ok());
        };
        Arbiter::current().spawn(exe);
        System::current().stop();
        system.run().unwrap();
    }

    #[traced_test]
    #[test]
    fn actix_handler_test() {
        let system = System::new();
        let exe = async {
            let task_bmc = TaskBmc::new();
            let addr = task_bmc.start();
            assert!(addr.send(Ping).await.unwrap().is_ok());
            let ret = addr.send(Create("http://bilibili.com")).await.unwrap();
            assert!(ret.is_ok());
            let id = ret.unwrap();
            assert!(addr.send(Cancel(id)).await.is_ok());
            assert!(addr.send(Revive(id)).await.is_ok());
            assert!(addr.send(Pause(id)).await.is_ok());
            assert!(addr.send(Continue_(id)).await.is_ok());
            assert!(addr.send(Restart(id)).await.is_ok());
            assert!(addr.send(Remove(id)).await.is_ok());
        };
        Arbiter::current().spawn(exe);
        System::current().stop();
        system.run().unwrap();
    }
}
