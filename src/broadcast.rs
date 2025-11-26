use std::{
    collections::{HashMap, HashSet},
    mem::take,
    ops::Deref,
    time::Duration,
};

const TIMER_DURATION: Duration = Duration::from_millis(100);
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

use crate::{
    messages::Message,
    nodes::{MessageType, Node, NodeState},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum BroadcastPayload {
    Broadcast {
        #[serde(rename = "message")]
        msg: u32,
    },
    BroadcastOk,
    Gossip {
        #[serde(rename = "message")]
        msg: HashSet<u32>,
    },
    Read,
    ReadOk {
        messages: HashSet<u32>,
    },
    Topology {
        topology: HashMap<String, Vec<String>>,
    },
    TopologyOk,
}

pub struct BroadcastTimer {}

pub struct BroadcastNode {
    state: NodeState<BroadcastTimer>,
    messages: HashSet<u32>,
}
impl Node for BroadcastNode {
    type PayloadType = BroadcastPayload;
    type Timer = BroadcastTimer;

    fn init() -> Self {
        let mut res = Self {
            state: NodeState::init(),
            messages: HashSet::default(),
        };
        res.queue_timer(BroadcastTimer {}, TIMER_DURATION);
        res
    }

    fn process_message(&mut self, message: MessageType<Self::PayloadType>) -> anyhow::Result<()> {
        if let MessageType::Defined(message) = message {
            match message.body.payload {
                Self::PayloadType::Broadcast { msg } => {
                    self.messages.insert(msg);
                    let payload = BroadcastPayload::BroadcastOk;
                    self.write_message(&payload, message.body.id, message.src)?;
                }
                Self::PayloadType::Read => {
                    let payload = BroadcastPayload::ReadOk {
                        messages: self.messages.clone(),
                    };
                    self.write_message(&payload, message.body.id, message.src)?;
                }
                Self::PayloadType::Gossip { msg } => {
                    self.messages.extend(msg);
                    // self.write_message(payload, message.body.id, message.src)?;
                }
                Self::PayloadType::Topology { .. } => {
                    let payload = BroadcastPayload::TopologyOk;
                    self.write_message(&payload, message.body.id, message.src)?;
                }
                Self::PayloadType::BroadcastOk { .. } => {}
                Self::PayloadType::TopologyOk => unreachable!(),
                Self::PayloadType::ReadOk { .. } => unreachable!(),
            }
        }
        Ok(())
    }

    fn get_state(&self) -> &NodeState<BroadcastTimer> {
        &self.state
    }

    fn get_state_mut(&mut self) -> &mut NodeState<BroadcastTimer> {
        &mut self.state
    }

    fn handle_timer(&mut self, timer: Self::Timer) -> anyhow::Result<()> {
        let mut rand = rand::rng();
        let mut nodes = self.state.nodes_ids.clone();
        nodes.shuffle(&mut rand);
        let mut i = 0;
        let resp = BroadcastPayload::Gossip {
            msg: take(&mut self.messages),
        };
        for node in nodes {
            if i >= 4 {
                break;
            }
            self.write_message(&resp, None, node)?;
            i += 1;
        }
        if let BroadcastPayload::Gossip { msg } = resp {
            self.messages = msg;
        }
        self.queue_timer(BroadcastTimer {}, TIMER_DURATION)?;
        Ok(())
    }
}
