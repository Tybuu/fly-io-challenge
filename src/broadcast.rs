use std::{
    collections::{HashMap, HashSet},
    ops::Deref,
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
        Self {
            state: NodeState::init(),
            messages: HashSet::default(),
        }
    }

    fn process_message(&mut self, message: Message<Self::PayloadType>) -> anyhow::Result<()> {
        match message.body.payload {
            Self::PayloadType::Broadcast { msg } => {
                self.messages.insert(msg);
                let payload = BroadcastPayload::BroadcastOk;
                self.write_message(payload, message.body.id, message.src)?;

                let resp = BroadcastPayload::Gossip { msg };

                for node in self.state.nodes_ids.clone() {
                    if node != self.state.node_id {
                        self.write_message(resp.clone(), None, node)?;
                    }
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
        Ok(())
    }
}
