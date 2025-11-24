use std::{
    fmt::{self, Debug},
    io::{StdoutLock, Write},
};

use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, BufReader};

use crate::messages::{Body, Message};

pub trait NodeState: Sized {
    type PayloadType: Serialize + for<'de> Deserialize<'de> + Clone + Debug;

    fn init() -> Self;
    fn process_message(
        &mut self,
        message: Message<Self::PayloadType>,
    ) -> Result<MessageResponse<Self::PayloadType>, MessageError>;
}

pub enum MessageResponse<P: Serialize + for<'de> Deserialize<'de> + Clone + Debug> {
    Init {
        node_id: String,
        node_ids: Vec<String>,
        payload: P,
    },
    Response {
        payload: P,
    },
}

#[derive(Debug)]
pub struct MessageError;

impl fmt::Display for MessageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Message Error!")
    }
}
impl std::error::Error for MessageError {}

pub struct Node<S: NodeState> {
    node_id: String,
    nodes_ids: Vec<String>,
    tx_id: usize,
    stdout: StdoutLock<'static>,
    state: S,
}

impl<S: NodeState> Node<S> {
    pub fn init() -> Self {
        Self {
            node_id: Default::default(),
            nodes_ids: Default::default(),
            tx_id: Default::default(),
            stdout: std::io::stdout().lock(),
            state: S::init(),
        }
    }

    pub async fn run(mut self) -> anyhow::Result<()> {
        let stdin = tokio::io::stdin();
        let mut reader = BufReader::new(stdin);
        loop {
            let mut buffer = String::new();
            reader.read_line(&mut buffer).await?;
            let msg: Message<S::PayloadType> = serde_json::from_str(&buffer)?;
            let msg_id = msg.body.id;
            let sender = msg.src.clone();
            let resp = self.state.process_message(msg)?;
            match resp {
                MessageResponse::Init {
                    node_id,
                    node_ids,
                    payload,
                } => {
                    self.node_id = node_id;
                    self.nodes_ids = node_ids;
                    self.tx_id = 1;
                    self.write_message(payload, msg_id, sender)?;
                }
                MessageResponse::Response { payload } => {
                    self.write_message(payload, msg_id, sender)?;
                }
            }
        }
    }

    fn write_message(
        &mut self,
        payload: S::PayloadType,
        req_id: Option<usize>,
        dst: String,
    ) -> anyhow::Result<()> {
        let msg = Message {
            src: self.node_id.clone(),
            dst,
            body: Body {
                id: Some(self.tx_id),
                req_id,
                payload,
            },
        };
        self.tx_id += 1;
        serde_json::to_writer(&mut self.stdout, &msg)?;
        self.stdout.write_all(b"\n")?;
        Ok(())
    }
}
