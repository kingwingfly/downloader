use super::error::ActorResult;
use crate::utils::TempDirHandler;
use actix::prelude::*;

use reqwest::Client;
use std::sync::Arc;
use tracing::debug;
use url::Url;

use tracing::{instrument, Level};

// region TaskActor
#[derive(Debug)]
pub struct TaskActor {
    paused: bool,
    client: Arc<Client>,
}

impl TaskActor {
    pub fn new() -> Self {
        #[cfg(test)]
        crate::config::config_init().unwrap();
        Self {
            paused: false,
            client: Arc::new(
                reqwest::Client::builder()
                    .user_agent(crate::config::get_config("user-agent").unwrap())
                    .build()
                    .unwrap(),
            ),
        }
    }
}

impl Actor for TaskActor {
    type Context = Context<Self>;
}

async fn get_total(client: Arc<Client>, url: Url) -> Option<usize> {
    client
        .get(url)
        .header("Referer", "https://www.bilibili.com/")
        .header("Range", "bytes=0-0".to_string())
        .send()
        .await
        .ok()?
        .headers()
        .get(reqwest::header::CONTENT_RANGE)?
        .to_str()
        .ok()?
        .split('/')
        .last()?
        .parse::<usize>()
        .ok()
}

// endregion TaskActor

//region RunTask Message

#[derive(Message)]
#[rtype(result = "ActorResult<()>")]
#[cfg_attr(test, derive(Debug))]
pub struct RunTask {
    suffix: String,
    url: Url,
    temp_dir: Arc<TempDirHandler>,
    finished: usize,
    total: usize,
    tx: tokio::sync::oneshot::Sender<ActorResult<()>>,
}

impl RunTask {
    pub fn new<S>(
        suffix: S,
        url: Url,
        temp_dir: Arc<TempDirHandler>,
        tx: tokio::sync::oneshot::Sender<ActorResult<()>>,
    ) -> Self
    where
        S: AsRef<str>,
    {
        Self {
            suffix: suffix.as_ref().to_string(),
            url,
            temp_dir,
            finished: 0,
            total: 0,
            tx,
        }
    }
}

impl Handler<RunTask> for TaskActor {
    type Result = ActorResult<()>;

    #[instrument(level=Level::DEBUG, skip(self, msg, ctx), fields(url=msg.url.as_str(), format=msg.suffix), err)]
    fn handle(&mut self, mut msg: RunTask, ctx: &mut Self::Context) -> Self::Result {
        match self.paused {
            true => {
                if !ctx.state().stopping() {
                    ctx.notify_later(msg, tokio::time::Duration::from_secs(2));
                }
            }
            false => {
                let client = self.client.clone();
                let addr = ctx.address();
                ctx.spawn(actix::fut::wrap_future(async move {
                    if msg.total == 0 {
                        msg.total = match get_total(client.clone(), msg.url.clone()).await {
                            Some(total) => total,
                            None => return,
                        }
                    }
                    let mut resp = match client
                        .get(msg.url.clone())
                        .header("Referer", "https://www.bilibili.com/")
                        .header(
                            "Range",
                            format!(
                                "bytes={}-{}",
                                msg.finished,
                                (msg.total - 1).min(msg.finished + 5 * (1 << 20))
                            ),
                        )
                        .send()
                        .await
                    {
                        Ok(resp) => resp,
                        Err(_) => return,
                    };
                    loop {
                        match resp.chunk().await {
                            Ok(Some(c)) => {
                                msg.temp_dir.write(&msg.suffix, &c).unwrap();
                            }
                            Ok(None) => break,
                            Err(_) => return,
                        }
                    }
                    msg.finished += 5 * (1 << 20) + 1;
                    #[cfg(test)]
                    debug!("{}", resp.status());
                    match msg.finished >= msg.total {
                        false => addr.do_send(msg), // move msg to next epoch
                        true => msg.tx.send(Ok(())).unwrap(), // drop msg -> drop Arc<TempDirHandler> -> if strong count is 0,
                                                              // then merge will invoke when drop TempDirHandler
                    }
                }));
            }
        }
        Ok(())
    }
}

//endregion RunTask Message

// region Pause Message
#[derive(Message)]
#[rtype(result = "ActorResult<()>")]
pub struct Pause;

impl Handler<Pause> for TaskActor {
    type Result = ActorResult<()>;

    fn handle(&mut self, _msg: Pause, _ctx: &mut Self::Context) -> Self::Result {
        if !self.paused {
            debug!("pause");
            self.paused = true;
        }
        Ok(())
    }
}

// endregion Pause Message

// region Continue Meassage

#[derive(Message)]
#[rtype(result = "ActorResult<()>")]
pub struct Continue_;

impl Handler<Continue_> for TaskActor {
    type Result = ActorResult<()>;

    fn handle(&mut self, _msg: Continue_, _ctx: &mut Self::Context) -> Self::Result {
        if self.paused {
            debug!("continue");
            self.paused = false;
        }
        Ok(())
    }
}

// endregion Continue Meassage

// region Cancel Message

#[derive(Message)]
#[rtype(result = "ActorResult<()>")]
pub struct Cancel;

impl Handler<Cancel> for TaskActor {
    type Result = ActorResult<()>;

    fn handle(&mut self, _msg: Cancel, ctx: &mut Self::Context) -> Self::Result {
        ctx.stop();
        Ok(())
    }
}

#[cfg(test)]
#[derive(Message, Default)]
#[rtype(result = "()")]
pub struct HeartBeat {
    count: usize,
}

#[cfg(test)]
impl Drop for HeartBeat {
    fn drop(&mut self) {
        debug!("heat beat {}", self.count);
    }
}

#[cfg(test)]
impl Handler<HeartBeat> for TaskActor {
    type Result = ();

    fn handle(&mut self, msg: HeartBeat, ctx: &mut Self::Context) -> Self::Result {
        if self.paused {
            ctx.notify_later(msg, tokio::time::Duration::from_secs(2));
            return;
        }
        let addr = ctx.address();
        ctx.spawn(actix::fut::wrap_future(async move {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            addr.do_send(HeartBeat {
                count: msg.count + 1,
            });
        }));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tracing_test::traced_test]
    #[actix_rt::test]
    async fn heat_beat_test() {
        let addr = TaskActor::new().start();
        addr.do_send(HeartBeat::default());
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
        addr.do_send(Pause);
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        addr.do_send(Continue_);
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }

    #[tracing_test::traced_test]
    #[actix_rt::test]
    async fn run_task_test() {
        let addr = TaskActor::new().start();
        let temp_dir = Arc::new(TempDirHandler::new("file").unwrap());
        let (tx, rx) = tokio::sync::oneshot::channel();
        let run_task = RunTask::new(
            "mp4",
            Url::parse("https://upos-sz-mirror08c.bilivideo.com/upgcxcode/66/77/1049107766/1049107766-1-30112.m4s?e=ig8euxZM2rNcNbdlhoNvNC8BqJIzNbfqXBvEqxTEto8BTrNvN0GvT90W5JZMkX_YN0MvXg8gNEV4NC8xNEV4N03eN0B5tZlqNxTEto8BTrNvNeZVuJ10Kj_g2UB02J0mN0B5tZlqNCNEto8BTrNvNC7MTX502C8f2jmMQJ6mqF2fka1mqx6gqj0eN0B599M=&uipk=5&nbs=1&deadline=1698616254&gen=playurlv2&os=08cbv&oi=3736210139&trid=db65754bb9494698aa13ec17f376d111u&mid=32280488&platform=pc&upsig=a8b17c487797cac95a5fc6e967f81eaf&uparams=e,uipk,nbs,deadline,gen,os,oi,trid,mid,platform&bvc=vod&nettype=0&orderid=0,3&buvid=&build=0&f=u_0_0&agrr=1&bw=669180&logo=80000000").unwrap(),
            temp_dir.clone(),tx
        );
        assert!(addr.send(run_task).await.is_ok());
        assert!(rx.await.is_ok());
        let (tx, rx) = tokio::sync::oneshot::channel();
        let run_task = RunTask::new(
            "aac",
            Url::parse("https://upos-sz-mirrorali.bilivideo.com/upgcxcode/66/77/1049107766/1049107766-1-30280.m4s?e=ig8euxZM2rNcNbdlhoNvNC8BqJIzNbfqXBvEqxTEto8BTrNvN0GvT90W5JZMkX_YN0MvXg8gNEV4NC8xNEV4N03eN0B5tZlqNxTEto8BTrNvNeZVuJ10Kj_g2UB02J0mN0B5tZlqNCNEto8BTrNvNC7MTX502C8f2jmMQJ6mqF2fka1mqx6gqj0eN0B599M=&uipk=5&nbs=1&deadline=1698616254&gen=playurlv2&os=alibv&oi=3736210139&trid=db65754bb9494698aa13ec17f376d111u&mid=32280488&platform=pc&upsig=7a99aaee8fa3f4466c1fe804770f3264&uparams=e,uipk,nbs,deadline,gen,os,oi,trid,mid,platform&bvc=vod&nettype=0&orderid=0,3&buvid=&build=0&f=u_0_0&agrr=1&bw=30625&logo=80000000").unwrap(),
            temp_dir.clone(),tx
        );
        assert!(addr.send(run_task).await.is_ok());
        assert!(rx.await.is_ok());
        tokio::time::sleep(tokio::time::Duration::from_secs(15)).await;
    }
}
