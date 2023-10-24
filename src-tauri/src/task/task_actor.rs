use actix::prelude::*;
use tracing::debug;
use url::Url;

use super::error::ActorResult;

// region TaskActor
#[derive(Debug)]
pub struct TaskActor {
    paused: bool,
}

impl TaskActor {
    pub fn new() -> Self {
        Self { paused: false }
    }
}

impl Actor for TaskActor {
    type Context = Context<Self>;
}

#[derive(Message)]
#[rtype(result = "ActorResult<()>")]
pub struct RunTask {
    pub url: Url,
}

impl Handler<RunTask> for TaskActor {
    type Result = ActorResult<()>;

    fn handle(&mut self, msg: RunTask, _ctx: &mut Self::Context) -> Self::Result {
        debug!("actix saving {}", msg.url.to_string());
        Ok(())
    }
}

#[derive(Message)]
#[rtype(result = "ActorResult<()>")]
pub struct Pause;

impl Handler<Pause> for TaskActor {
    type Result = ActorResult<()>;

    fn handle(&mut self, msg: Pause, ctx: &mut Self::Context) -> Self::Result {
        if !self.paused {
            debug!("pause");
            self.paused = true;
        }
        Ok(())
    }
}

#[derive(Message)]
#[rtype(result = "ActorResult<()>")]
pub struct Continue_;

impl Handler<Continue_> for TaskActor {
    type Result = ActorResult<()>;

    fn handle(&mut self, _msg: Continue_, ctx: &mut Self::Context) -> Self::Result {
        if self.paused {
            debug!("continue");
            self.paused = false;
        }
        Ok(())
    }
}

#[cfg(test)]
#[derive(Message)]
#[rtype(result = "()")]
pub struct HeartBeat;

#[cfg(test)]
impl Drop for HeartBeat {
    fn drop(&mut self) {
        debug!("heat beat");
    }
}

#[cfg(test)]
impl Handler<HeartBeat> for TaskActor {
    type Result = ();

    fn handle(&mut self, msg: HeartBeat, ctx: &mut Self::Context) -> Self::Result {
        let addr = ctx.address();
        if self.paused {
            Arbiter::current().spawn(async move {
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                addr.do_send(msg);
            });
            return;
        }
        Arbiter::current().spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            addr.do_send(HeartBeat);
        });
    }
}
// endregion TaskActor

#[cfg(test)]
mod tests {
    use super::*;

    #[tracing_test::traced_test]
    #[actix_rt::test]
    async fn heat_beat_test() {
        actix::Arbiter::current().spawn(async {
            let addr = TaskActor::new().start();
            addr.do_send(HeartBeat);
            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
            addr.do_send(Pause);
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            addr.do_send(Continue_);
        });
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
    }
}
