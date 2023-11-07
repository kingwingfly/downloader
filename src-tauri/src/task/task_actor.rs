use super::error::{actor_error, ActorResult};
use crate::utils::TempDirHandler;

use actix::prelude::*;
use num_enum::{FromPrimitive, IntoPrimitive};
use reqwest::Client;
use std::sync::atomic::{AtomicU8, AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::oneshot;
use tracing::{instrument, Level};
use url::Url;

#[derive(Eq, PartialEq, FromPrimitive, IntoPrimitive)]
#[repr(u8)]
enum Instrument {
    #[num_enum(default)]
    Continue = 0,
    TryPause,
    Paused,
    Cancel,
    Finish,
}

#[derive(Debug, Eq, PartialEq, FromPrimitive, IntoPrimitive)]
#[repr(u8)]
enum State {
    #[num_enum(default)]
    Downloading = 0,
    Pausing,
    Paused,
    Cancelled,
    Finished,
}

#[derive(Default)]
struct TaskState {
    state: AtomicU8,
}

impl TaskState {
    fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    fn now(&self) -> State {
        self.state.load(Ordering::Relaxed).into()
    }

    fn trans(&self, instrument: Instrument) {
        match instrument {
            x => self.state.store(x.into(), Ordering::Relaxed),
        }
    }
}

// region TaskActor
pub struct TaskActor {
    state: Arc<TaskState>,
    total: Arc<AtomicUsize>,
    finished: Arc<AtomicUsize>,
    filename: Option<String>,
}

impl TaskActor {
    pub fn new() -> Self {
        Self {
            state: Arc::new(TaskState::new()),
            total: Arc::new(AtomicUsize::new(0)),
            finished: Arc::new(AtomicUsize::new(0)),
            filename: None,
        }
    }
}

impl Actor for TaskActor {
    type Context = Context<Self>;
}

// endregion TaskActor

//region RunTask Message

#[derive(Message)]
#[rtype(result = "ActorResult<()>")]
#[cfg_attr(test, derive(Debug))]
pub struct RunTask {
    suffix: String,
    url: Url,
    referer: String,
    temp_dir: Arc<TempDirHandler>,
    tx: oneshot::Sender<ActorResult<()>>,
}

impl RunTask {
    pub fn new<S1, S2>(
        suffix: S1,
        url: Url,
        referer: S2,
        temp_dir: Arc<TempDirHandler>,
        tx: oneshot::Sender<ActorResult<()>>,
    ) -> Self
    where
        S1: AsRef<str>,
        S2: AsRef<str>,
    {
        Self {
            suffix: suffix.as_ref().to_string(),
            url,
            referer: referer.as_ref().to_string(),
            temp_dir,
            tx,
        }
    }
}

impl Handler<RunTask> for TaskActor {
    type Result = ActorResult<()>;

    #[instrument(level=Level::DEBUG, skip(self, msg, _ctx), fields(url=msg.url.as_str(), format=msg.suffix), err)]
    fn handle(&mut self, msg: RunTask, _ctx: &mut Self::Context) -> Self::Result {
        let actor_total = self.total.clone();
        let actor_finished = self.finished.clone();
        let state = self.state.clone();
        actix_rt::spawn(async move {
            let client = Arc::new(
                reqwest::Client::builder()
                    .user_agent(crate::config::get_config("user-agent").unwrap())
                    .build()
                    .unwrap(),
            );
            let total = get_total(client.clone(), msg.url.clone(), &msg.referer)
                .await
                .unwrap_or(0);
            actor_total.fetch_add(total, Ordering::Relaxed);
            let mut finished: usize = 0;
            while finished < total {
                match state.now() {
                    State::Downloading => {
                        let mut resp = client
                            .get(msg.url.clone())
                            .header("Referer", &msg.referer)
                            .header(
                                "Range",
                                format!(
                                    "bytes={}-{}",
                                    finished,
                                    (total - 1).min(finished + (1 << 23))
                                ),
                            )
                            .send()
                            .await
                            .unwrap();
                        while let Some(c) = resp.chunk().await.unwrap() {
                            msg.temp_dir.write(&msg.suffix, &c).unwrap();
                            actor_finished.fetch_add(c.len(), Ordering::Relaxed);
                        }
                        finished += (1 << 23) + 1;
                    }
                    State::Cancelled => break,
                    _ => {
                        state.trans(Instrument::Paused);
                        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                    }
                }
            }
            if state.now() == State::Cancelled {
                msg.tx.send(actor_error::Cancelled.fail()).unwrap();
            } else {
                if actor_finished.load(Ordering::Relaxed) >= actor_total.load(Ordering::Relaxed) {
                    state.trans(Instrument::Finish);
                }
                msg.tx.send(Ok(())).unwrap();
            }
        });
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
        self.state.trans(Instrument::TryPause);
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
        self.state.trans(Instrument::Continue);
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

    fn handle(&mut self, _msg: Cancel, _ctx: &mut Self::Context) -> Self::Result {
        self.state.trans(Instrument::Cancel);
        Ok(())
    }
}

// endregion Cancel Message

// region Revive Message

#[derive(Message)]
#[rtype(result = "ActorResult<()>")]
pub struct Revive;

impl Handler<Revive> for TaskActor {
    type Result = ActorResult<()>;

    fn handle(&mut self, _msg: Revive, _ctx: &mut Self::Context) -> Self::Result {
        todo!()
    }
}

// endregion Revive Message

// region Restart Message

#[derive(Message)]
#[rtype(result = "ActorResult<()>")]
pub struct Restart;

impl Handler<Restart> for TaskActor {
    type Result = ActorResult<()>;

    fn handle(&mut self, _msg: Restart, _ctx: &mut Self::Context) -> Self::Result {
        todo!()
    }
}

// endregion Restart Message

// region ProgressQuery Message
#[derive(Message)]
#[rtype(result = "ActorResult<()>")]
pub struct ProgressQuery {
    tx: oneshot::Sender<ActorResult<(String, usize, usize, String)>>,
}

impl ProgressQuery {
    pub fn new(tx: oneshot::Sender<ActorResult<(String, usize, usize, String)>>) -> Self {
        Self { tx }
    }
}

impl Handler<ProgressQuery> for TaskActor {
    type Result = ActorResult<()>;

    fn handle(&mut self, msg: ProgressQuery, _ctx: &mut Self::Context) -> Self::Result {
        let finished = self.finished.load(Ordering::Relaxed);
        let total = self.total.load(Ordering::Relaxed);
        let state = match self.state.now() {
            State::Downloading => "downloading",
            State::Pausing => "pausing",
            State::Paused => "paused",
            State::Cancelled => "cancelled",
            State::Finished => "finished",
        };
        msg.tx
            .send(Ok((
                self.filename.as_deref().unwrap_or("unknown").to_owned(),
                finished,
                total,
                state.to_string(),
            )))
            .unwrap();
        Ok(())
    }
}

// endregion ProgressQuery Message

// region SetFilename Message

#[derive(Message)]
#[rtype(result = "ActorResult<()>")]
pub struct SetFilename(pub String);

impl Handler<SetFilename> for TaskActor {
    type Result = ActorResult<()>;

    fn handle(&mut self, msg: SetFilename, _ctx: &mut Self::Context) -> Self::Result {
        self.filename = Some(msg.0);
        Ok(())
    }
}

// endregion SetFilename Message

async fn get_total(client: Arc<Client>, url: Url, referer: &str) -> Option<usize> {
    client
        .get(url)
        .header("Referer", referer)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tracing_test::traced_test]
    #[actix_rt::test]
    async fn run_task_test() {
        crate::config::config_init().unwrap();
        let addr = TaskActor::new().start();
        let temp_dir = Arc::new(TempDirHandler::new("file").unwrap());
        let (tx, rx) = tokio::sync::oneshot::channel();
        let run_task = RunTask::new(
            "mp4",
            Url::parse("https://upos-sz-mirror08c.bilivideo.com/upgcxcode/66/77/1049107766/1049107766-1-30112.m4s?e=ig8euxZM2rNcNbdlhoNvNC8BqJIzNbfqXBvEqxTEto8BTrNvN0GvT90W5JZMkX_YN0MvXg8gNEV4NC8xNEV4N03eN0B5tZlqNxTEto8BTrNvNeZVuJ10Kj_g2UB02J0mN0B5tZlqNCNEto8BTrNvNC7MTX502C8f2jmMQJ6mqF2fka1mqx6gqj0eN0B599M=&uipk=5&nbs=1&deadline=1698616254&gen=playurlv2&os=08cbv&oi=3736210139&trid=db65754bb9494698aa13ec17f376d111u&mid=32280488&platform=pc&upsig=a8b17c487797cac95a5fc6e967f81eaf&uparams=e,uipk,nbs,deadline,gen,os,oi,trid,mid,platform&bvc=vod&nettype=0&orderid=0,3&buvid=&build=0&f=u_0_0&agrr=1&bw=669180&logo=80000000").unwrap(),
            "https://www.bilibili.com/",
            temp_dir.clone(),tx
        );
        assert!(addr.send(run_task).await.is_ok());
        assert!(rx.await.is_ok());
        let (tx, rx) = tokio::sync::oneshot::channel();
        let run_task = RunTask::new(
            "aac",
            Url::parse("https://upos-sz-mirrorali.bilivideo.com/upgcxcode/66/77/1049107766/1049107766-1-30280.m4s?e=ig8euxZM2rNcNbdlhoNvNC8BqJIzNbfqXBvEqxTEto8BTrNvN0GvT90W5JZMkX_YN0MvXg8gNEV4NC8xNEV4N03eN0B5tZlqNxTEto8BTrNvNeZVuJ10Kj_g2UB02J0mN0B5tZlqNCNEto8BTrNvNC7MTX502C8f2jmMQJ6mqF2fka1mqx6gqj0eN0B599M=&uipk=5&nbs=1&deadline=1698616254&gen=playurlv2&os=alibv&oi=3736210139&trid=db65754bb9494698aa13ec17f376d111u&mid=32280488&platform=pc&upsig=7a99aaee8fa3f4466c1fe804770f3264&uparams=e,uipk,nbs,deadline,gen,os,oi,trid,mid,platform&bvc=vod&nettype=0&orderid=0,3&buvid=&build=0&f=u_0_0&agrr=1&bw=30625&logo=80000000").unwrap(),
            "https://www.bilibili.com/",
            temp_dir.clone(),tx
        );
        assert!(addr.send(run_task).await.is_ok());
        assert!(rx.await.is_ok());
        tokio::time::sleep(tokio::time::Duration::from_secs(15)).await;
    }

    #[test]
    fn state_test() {
        let state = TaskState::new();
        assert_eq!(state.now(), State::Downloading);
        state.trans(Instrument::TryPause);
        assert_eq!(state.now(), State::Pausing);
        state.trans(Instrument::Paused);
        assert_eq!(state.now(), State::Paused);
        state.trans(Instrument::Finish);
        assert_eq!(state.now(), State::Finished);
    }
}
