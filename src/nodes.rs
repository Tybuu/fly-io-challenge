use std::{
    cmp::{Ordering, Reverse},
    collections::BinaryHeap,
    fmt::{self, Debug},
    io::{StdoutLock, Write},
    time::Duration,
};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    select,
    sync::mpsc,
    time::Instant,
};

use crate::{
    messages::{Body, InitPayload, Message},
    seqkv::SeqPayload,
};

struct TimerState<T> {
    state: T,
    expires_at: Instant,
}

impl<T> PartialEq for TimerState<T> {
    fn eq(&self, other: &Self) -> bool {
        self.expires_at == other.expires_at
    }
}

impl<T> Eq for TimerState<T> {}

impl<T> PartialOrd for TimerState<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.expires_at.cmp(&other.expires_at))
    }
}

impl<T> Ord for TimerState<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.expires_at.cmp(&other.expires_at)
    }
}

pub struct NodeState<T> {
    pub node_id: String,
    pub nodes_ids: Vec<String>,
    tx_id: usize,
    queue: BinaryHeap<Reverse<TimerState<T>>>,
    stdout: StdoutLock<'static>,
}

impl<T> NodeState<T> {
    pub fn init() -> NodeState<T> {
        NodeState {
            node_id: String::default(),
            nodes_ids: Vec::default(),
            tx_id: 0,
            queue: BinaryHeap::default(),
            stdout: std::io::stdout().lock(),
        }
    }
}

pub enum MessageType<T> {
    Defined(Message<T>),
    Seq(Message<SeqPayload>),
}

pub trait Node: Sized {
    type PayloadType: Serialize + for<'de> Deserialize<'de> + Clone + Debug;
    type Timer;

    fn init() -> Self;

    fn get_state(&self) -> &NodeState<Self::Timer>;

    fn get_state_mut(&mut self) -> &mut NodeState<Self::Timer>;

    fn process_message(&mut self, message: MessageType<Self::PayloadType>) -> anyhow::Result<()>;

    fn handle_timer(&mut self, timer: Self::Timer) -> anyhow::Result<()>;

    fn queue_timer(&mut self, timer: Self::Timer, dur: Duration) -> anyhow::Result<()> {
        let time = Instant::now() + dur;
        let res = self.get_state_mut();
        res.queue.push(Reverse(TimerState {
            state: timer,
            expires_at: time,
        }));
        Ok(())
    }

    async fn run(mut self) -> anyhow::Result<()> {
        let (tx, mut rx) = mpsc::channel(100);
        let handle = tokio::spawn(async move {
            let stdin = tokio::io::stdin();
            let mut reader = BufReader::new(stdin);
            loop {
                let mut buf = String::new();
                if (reader.read_line(&mut buf).await).is_ok() && tx.send(buf).await.is_err() {
                    break;
                }
            }
        });
        loop {
            let min = self.get_state_mut().queue.pop();
            let mut buf = None;
            if let Some(min) = min {
                select! {
                    _ = tokio::time::sleep_until(min.0.expires_at) => {
                        self.handle_timer(min.0.state)?;
                        continue;
                    },
                    buffer = rx.recv() => {
                        self.get_state_mut().queue.push(min);
                        buf = buffer;
                    }
                }
            }
            let buf = if let Some(buffer) = buf {
                buffer
            } else {
                rx.recv().await.unwrap()
            };
            let values: Map<String, Value> = serde_json::from_str(&buf)?;
            if let Some(Value::String(src)) = values.get("src") {
                if src == "seq-kv" {
                    let res = serde_json::from_str::<Message<SeqPayload>>(&buf);
                    if let Ok(msg) = res {
                        self.process_message(MessageType::Seq(msg))?;
                    } else {
                        res?;
                    }
                    continue;
                }
            }
            let res = serde_json::from_str::<Message<Self::PayloadType>>(&buf);
            if let Ok(msg) = res {
                self.process_message(MessageType::Defined(msg))?;
            } else if let Ok(init) = serde_json::from_str::<Message<InitPayload>>(&buf) {
                let state = self.get_state_mut();
                match init.body.payload {
                    InitPayload::Init { node_id, node_ids } => {
                        state.node_id = node_id;
                        state.nodes_ids = node_ids;
                        state.tx_id = 1;
                        let payload = InitPayload::InitOk;
                        self.write_message(&payload, init.body.id, init.src)?;
                    }
                    InitPayload::InitOk => {}
                }
            } else {
                res?;
            }
        }
    }

    fn write_message<P: Serialize>(
        &mut self,
        payload: &P,
        req_id: Option<usize>,
        dst: String,
    ) -> anyhow::Result<()> {
        let state = self.get_state_mut();
        let msg = Message {
            src: state.node_id.clone(),
            dst,
            body: Body {
                id: Some(state.tx_id),
                req_id,
                payload,
            },
        };
        state.tx_id += 1;
        serde_json::to_writer(&mut state.stdout, &msg)?;
        state.stdout.write_all(b"\n")?;
        Ok(())
    }
}
