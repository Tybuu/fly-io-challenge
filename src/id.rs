use serde::{Deserialize, Serialize};

use crate::{
    messages::Message,
    nodes::{MessageError, MessageResponse, NodeState},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum IdPayload {
    Init {
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk {},
    Generate {},
    GenerateOk {
        id: u64,
    },
}
pub struct IdState {
    current_id: u64,
}

impl NodeState for IdState {
    type PayloadType = IdPayload;

    fn init() -> Self {
        Self {
            current_id: rand::random(),
        }
    }

    fn process_message(
        &mut self,
        message: Message<Self::PayloadType>,
    ) -> Result<MessageResponse<Self::PayloadType>, MessageError> {
        match message.body.payload {
            Self::PayloadType::Init { node_id, node_ids } => Ok(MessageResponse::Init {
                node_id,
                node_ids,
                payload: Self::PayloadType::InitOk {},
            }),
            Self::PayloadType::Generate {} => {
                self.current_id += 1;
                Ok(MessageResponse::Response {
                    payload: Self::PayloadType::GenerateOk {
                        id: self.current_id - 1,
                    },
                })
            }
            Self::PayloadType::InitOk { .. } => unreachable!(),
            Self::PayloadType::GenerateOk { .. } => unreachable!(),
        }
    }
}
