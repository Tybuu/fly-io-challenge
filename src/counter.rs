use std::{collections::VecDeque, mem::take, time::Duration};

use serde::{Deserialize, Serialize};

use crate::{
    messages::Message,
    nodes::{MessageType, Node, NodeState},
    seqkv::SeqPayload,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum CounterPayload {
    Add { delta: u32 },
    AddOk {},
    Read {},
    ReadOk { value: u32 },
}

const KEY: &str = "reddit";
pub struct CounterTimer {}

const TIMER_DURATION: Duration = Duration::from_millis(500);

pub struct CounterNode {
    state: NodeState<CounterTimer>,
    current_state: u32,
    proposed_state: u32,
    adders: Vec<(String, Option<usize>)>,
    add_ops: u32,
}
impl Node for CounterNode {
    type PayloadType = CounterPayload;
    type Timer = CounterTimer;

    fn init() -> Self {
        let mut res = Self {
            state: NodeState::init(),
            current_state: 0,
            proposed_state: 0,
            adders: Vec::default(),
            add_ops: 0,
        };
        res.queue_timer(CounterTimer {}, TIMER_DURATION);
        res
    }

    fn handle_timer(&mut self, timer: Self::Timer) -> anyhow::Result<()> {
        let payload = SeqPayload::Read { key: KEY.into() };
        self.write_message(&payload, None, "seq-kv".into())?;
        self.queue_timer(CounterTimer {}, TIMER_DURATION)?;
        Ok(())
    }

    fn process_message(&mut self, message: MessageType<Self::PayloadType>) -> anyhow::Result<()> {
        if let MessageType::Defined(message) = message {
            match message.body.payload {
                CounterPayload::Add { delta } => {
                    self.add_ops += delta;
                    self.adders.push((message.src, message.body.id));
                    if self.adders.len() == 1 {
                        self.proposed_state = self.current_state + self.add_ops;
                        let msg = SeqPayload::Cas {
                            key: KEY.into(),
                            from: self.current_state,
                            to: self.proposed_state,
                        };
                        self.write_message(&msg, None, "seq-kv".into())?;
                    }
                }
                CounterPayload::AddOk {} => unreachable!(),
                CounterPayload::Read {} => {
                    let payload = CounterPayload::ReadOk {
                        value: self.current_state,
                    };
                    self.write_message(&payload, message.body.id, message.src.clone())?;
                }
                CounterPayload::ReadOk { .. } => unreachable!(),
            }
        } else if let MessageType::Seq(message) = message {
            match message.body.payload {
                SeqPayload::Read { .. } => unreachable!(),
                SeqPayload::ReadOk { value } => {
                    self.current_state = value;
                    if !self.adders.is_empty() {
                        self.proposed_state = self.current_state + self.add_ops;
                        let msg = SeqPayload::Cas {
                            key: KEY.into(),
                            from: self.current_state,
                            to: self.proposed_state,
                        };
                        self.write_message(&msg, None, "seq-kv".into())?;
                    }
                }
                SeqPayload::Write { .. } => unreachable!(),
                SeqPayload::WriteOk {} => {
                    self.proposed_state = self.current_state + self.add_ops;
                    let msg = SeqPayload::Cas {
                        key: KEY.into(),
                        from: self.current_state,
                        to: self.proposed_state,
                    };
                    self.write_message(&msg, None, "seq-kv".into())?;
                }
                SeqPayload::Cas { .. } => unreachable!(),
                SeqPayload::CasOk {} => {
                    self.current_state = self.proposed_state;
                    self.add_ops = 0;
                    self.proposed_state = self.current_state;
                    let adders = take(&mut self.adders);
                    let payload = CounterPayload::AddOk {};
                    for (adder, msg_id) in adders {
                        self.write_message(&payload, msg_id, adder)?;
                    }
                }
                SeqPayload::Error { code, .. } => {
                    if code == 20 && self.state.node_id == self.state.nodes_ids[0] {
                        let payload = SeqPayload::Write {
                            key: KEY.into(),
                            value: 0,
                        };
                        self.write_message(&payload, None, "seq-kv".into())?;
                    } else if code == 22 {
                        let payload = SeqPayload::Read { key: KEY.into() };
                        self.write_message(&payload, None, "seq-kv".into())?;
                    }
                }
            }
        }
        Ok(())
    }

    fn get_state(&self) -> &NodeState<CounterTimer> {
        &self.state
    }

    fn get_state_mut(&mut self) -> &mut NodeState<CounterTimer> {
        &mut self.state
    }
}
