use serde::{Deserialize, Serialize};

use crate::{
    messages::Message,
    nodes::{MessageError, MessageResponse, NodeState},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum EchoPayload {
    Init {
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk,
    Echo {
        echo: String,
    },
    EchoOk {
        echo: String,
    },
}

pub struct EchoState {}
impl NodeState for EchoState {
    type PayloadType = EchoPayload;

    fn init() -> Self {
        Self {}
    }

    fn process_message(
        &mut self,
        message: Message<Self::PayloadType>,
    ) -> Result<MessageResponse<Self::PayloadType>, MessageError> {
        match message.body.payload {
            Self::PayloadType::Init { node_id, node_ids } => Ok(MessageResponse::Init {
                node_id,
                node_ids,
                payload: Self::PayloadType::InitOk,
            }),
            Self::PayloadType::Echo { echo } => Ok(MessageResponse::Response {
                payload: Self::PayloadType::EchoOk { echo },
            }),
            Self::PayloadType::EchoOk { .. } => unreachable!(),
            Self::PayloadType::InitOk => unreachable!(),
        }
    }
}
