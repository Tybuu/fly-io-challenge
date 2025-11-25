use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::{
    messages::Message,
    nodes::{MessageError, MessageResponse, NodeState},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum BroadcastPayload {
    Init {
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk,
    Broadcast {
        message: u32,
    },
    BroadcastOk,
    Read,
    ReadOk {
        messages: HashSet<u32>,
    },
    Topology {
        topology: HashMap<String, Vec<String>>,
    },
    TopologyOk,
}

pub struct BroadcastState {
    messages: HashSet<u32>,
    node_id: String,
    nodes_ids: Vec<String>,
}
impl NodeState for BroadcastState {
    type PayloadType = BroadcastPayload;

    fn init() -> Self {
        Self {
            node_id: String::default(),
            nodes_ids: Vec::default(),
            messages: HashSet::default(),
        }
    }

    fn process_message(
        &mut self,
        message: Message<Self::PayloadType>,
    ) -> Result<MessageResponse<Self::PayloadType>, MessageError> {
        match message.body.payload {
            Self::PayloadType::Init { node_id, node_ids } => {
                self.node_id = node_id.clone();
                self.nodes_ids = node_ids.clone();
                Ok(MessageResponse::Init {
                    node_id,
                    node_ids,
                    payload: Self::PayloadType::InitOk,
                })
            }
            Self::PayloadType::Broadcast { message } => {
                self.messages.insert(message);
                Ok(MessageResponse::Response {
                    payload: Self::PayloadType::BroadcastOk,
                })
            }
            Self::PayloadType::Read => Ok(MessageResponse::Response {
                payload: Self::PayloadType::ReadOk {
                    messages: self.messages.clone(),
                },
            }),
            Self::PayloadType::Topology { .. } => {
                let res = 5;
                Ok(MessageResponse::Response {
                    payload: Self::PayloadType::TopologyOk,
                })
            }
            Self::PayloadType::BroadcastOk { .. } => unreachable!(),
            Self::PayloadType::InitOk => unreachable!(),
            Self::PayloadType::TopologyOk => unreachable!(),
            Self::PayloadType::ReadOk { .. } => unreachable!(),
        }
    }
}
