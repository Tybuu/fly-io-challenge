use std::{
    collections::{HashMap, HashSet},
    ops::Deref,
    time::Duration,
};

use serde::{Deserialize, Serialize};

use crate::{
    messages::Message,
    nodes::{Node, NodeState},
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
        msg: u32,
    },
    GossipOk {
        msg: u32,
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

pub struct BroadcastTimer {
    msg: u32,
}

pub struct BroadcastNode {
    state: NodeState<BroadcastTimer>,
    messages: HashSet<u32>,
    need_to_send: HashMap<u32, HashSet<String>>,
}
impl Node for BroadcastNode {
    type PayloadType = BroadcastPayload;
    type Timer = BroadcastTimer;

    fn init() -> Self {
        Self {
            state: NodeState::init(),
            messages: HashSet::default(),
            need_to_send: HashMap::default(),
        }
    }

    fn process_message(&mut self, message: Message<Self::PayloadType>) -> anyhow::Result<()> {
        match message.body.payload {
            Self::PayloadType::Broadcast { msg } => {
                let payload = BroadcastPayload::BroadcastOk;
                self.write_message(payload, message.body.id, message.src)?;

                let resp = BroadcastPayload::Gossip { msg };
                if self.messages.insert(msg) {
                    let mut nodes = HashSet::new();
                    for node in self.state.nodes_ids.clone() {
                        if node != self.state.node_id {
                            nodes.insert(node.clone());
                            self.write_message(resp.clone(), None, node)?;
                        }
                    }
                    self.need_to_send.insert(msg, nodes);
                    self.queue_timer(BroadcastTimer { msg }, Duration::from_millis(100))?;
                }
            }
            Self::PayloadType::Read => {
                let payload = BroadcastPayload::ReadOk {
                    messages: self.messages.clone(),
                };
                self.write_message(payload, message.body.id, message.src)?;
            }
            Self::PayloadType::Gossip { msg } => {
                self.messages.insert(msg);
                let payload = BroadcastPayload::GossipOk { msg };
                self.write_message(payload, message.body.id, message.src)?;
            }
            Self::PayloadType::GossipOk { msg } => {
                if let Some(set) = self.need_to_send.get_mut(&msg) {
                    set.remove(&message.src);
                    if set.is_empty() {
                        self.need_to_send.remove(&msg);
                    }
                }
            }
            Self::PayloadType::Topology { .. } => {
                let payload = BroadcastPayload::TopologyOk;
                self.write_message(payload, message.body.id, message.src)?;
            }
            Self::PayloadType::BroadcastOk { .. } => {}
            Self::PayloadType::TopologyOk => unreachable!(),
            Self::PayloadType::ReadOk { .. } => unreachable!(),
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
        if let Some(set) = self.need_to_send.get_mut(&timer.msg) {
            let resp = BroadcastPayload::Gossip { msg: timer.msg };
            for node in set.clone() {
                self.write_message(resp.clone(), None, node)?;
            }
            self.queue_timer(
                BroadcastTimer { msg: timer.msg },
                Duration::from_millis(100),
            )?;
        }
        Ok(())
    }
}
