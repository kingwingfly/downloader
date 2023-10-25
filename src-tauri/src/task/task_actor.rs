use actix::prelude::*;
use tracing::debug;
use url::Url;

use crate::utils::TempDirHandler;

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
    url: Url,
    temp_dir: TempDirHandler,
    finished: usize,
}

impl RunTask {
    pub fn new(url: Url) -> Self {
        Self {
            url,
            temp_dir: TempDirHandler::new().unwrap(),
            finished: 0,
        }
    }
}

impl Handler<RunTask> for TaskActor {
    type Result = ActorResult<()>;

    fn handle(&mut self, mut msg: RunTask, ctx: &mut Self::Context) -> Self::Result {
        debug!("actix saving {:.20}...", msg.url.to_string());
        let addr = ctx.address();
        match self.paused {
            true => {
                Arbiter::current().spawn(async move {
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                    addr.do_send(msg);
                });
            }
            false => {
                Arbiter::current().spawn(async move {
                    crate::config::config_init().unwrap();
                    let mut resp = reqwest::Client::builder()
                        .user_agent(crate::config::get_config("user-agent").unwrap())
                        .build()
                        .unwrap()
                        .get(msg.url)
                        .header("Referer", "https://www.bilibili.com/")
                        // .header("Range", "bytes=139334-283402")
                        .send()
                        .await
                        .unwrap();
                    debug!("{}", resp.status());
                    while let Some(c) = resp.chunk().await.unwrap() {
                        msg.temp_dir.write("file", &c).unwrap();
                        debug!("{} bytes written", c.len());
                        msg.finished += c.len();
                    }
                });
            }
        }
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
        Arbiter::current().spawn(async {
            let addr = TaskActor::new().start();
            addr.do_send(HeartBeat);
            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
            addr.do_send(Pause);
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            addr.do_send(Continue_);
        });
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
    }

    #[tracing_test::traced_test]
    #[actix_rt::test]
    async fn run_task_test() {
        let addr = TaskActor::new().start();
        let run_task = RunTask::new(Url::parse("https://upos-sz-mirror08c.bilivideo.com/upgcxcode/66/77/1049107766/1049107766-1-30112.m4s?e=ig8euxZM2rNcNbdlhoNvNC8BqJIzNbfqXBvEqxTEto8BTrNvN0GvT90W5JZMkX_YN0MvXg8gNEV4NC8xNEV4N03eN0B5tZlqNxTEto8BTrNvNeZVuJ10Kj_g2UB02J0mN0B5tZlqNCNEto8BTrNvNC7MTX502C8f2jmMQJ6mqF2fka1mqx6gqj0eN0B599M=&uipk=5&nbs=1&deadline=1698247626&gen=playurlv2&os=08cbv&oi=2073294230&trid=7a8edce76e8e4ec9a7b51760a1481264u&mid=32280488&platform=pc&upsig=2da912c6f773ad15e17858cadcebfd74&uparams=e,uipk,nbs,deadline,gen,os,oi,trid,mid,platform&bvc=vod&nettype=0&orderid=0,3&buvid=&build=0&f=u_0_0&agrr=0&bw=669180&logo=80000000").unwrap());
        assert!(addr.send(run_task).await.is_ok());
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
}
