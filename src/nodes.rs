use std::io::{self, StdinLock, StdoutLock, Write};

use serde_json::Deserializer;

use crate::messages::{Body, Message, Payload};

pub struct Node {
    node_id: String,
    nodes_ids: Vec<String>,
    tx_id: usize,
    stdout: StdoutLock<'static>,
}

impl Node {
    pub fn init() -> Self {
        Self {
            node_id: Default::default(),
            nodes_ids: Default::default(),
            tx_id: Default::default(),
            stdout: std::io::stdout().lock(),
        }
    }

    pub fn process_message(&mut self, message: Message) -> anyhow::Result<()> {
        match message.body.payload {
            Payload::Init { node_id, node_ids } => {
                self.node_id = node_id;
                self.nodes_ids = node_ids;
                self.tx_id = 2;
                let resp = Message {
                    src: self.node_id.clone(),
                    dst: message.src.clone(),
                    body: Body {
                        id: Some(1),
                        req_id: message.body.id,
                        payload: Payload::InitOk {},
                    },
                };
                serde_json::to_writer(&mut self.stdout, &resp)?;
                self.stdout.write_all(b"\n")?;
            }
            Payload::Echo { echo } => {
                let resp = Message {
                    src: self.node_id.clone(),
                    dst: message.src.clone(),
                    body: Body {
                        id: Some(self.tx_id),
                        req_id: message.body.id,
                        payload: Payload::EchoOk { echo },
                    },
                };
                self.tx_id += 1;
                serde_json::to_writer(&mut self.stdout, &resp)?;
                self.stdout.write_all(b"\n")?;
            }
            Payload::EchoOk { .. } => todo!(),
            Payload::InitOk {} => todo!(),
        }
        self.stdout.flush()?;
        Ok(())
    }
    pub fn run(mut self) -> anyhow::Result<()> {
        let stdin = std::io::stdin().lock();
        let stream = Deserializer::from_reader(stdin).into_iter::<Message>();
        for message in stream {
            let message = message?;
            self.process_message(message)?;
        }

        Ok(())
    }
}
