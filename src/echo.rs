use serde::{Deserialize, Serialize};

use crate::{
    messages::Message,
    nodes::{Node, NodeState},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum EchoPayload {
    Echo { echo: String },
    EchoOk { echo: String },
}

pub struct EchoTimer {}

pub struct EchoNode {
    state: NodeState<EchoTimer>,
}
impl Node for EchoNode {
    type PayloadType = EchoPayload;
    type Timer = EchoTimer;

    fn init() -> Self {
        Self {
            state: NodeState::init(),
        }
    }

    fn handle_timer(&mut self, timer: Self::Timer) -> anyhow::Result<()> {
        Ok(())
    }

    fn process_message(&mut self, message: Message<Self::PayloadType>) -> anyhow::Result<()> {
        match message.body.payload {
            EchoPayload::Echo { echo } => {
                let payload = EchoPayload::EchoOk { echo };
                self.write_message(payload, message.body.id, message.src)?;
            }
            Self::PayloadType::EchoOk { .. } => unreachable!(),
        }
        Ok(())
    }

    fn get_state(&self) -> &NodeState<EchoTimer> {
        &self.state
    }

    fn get_state_mut(&mut self) -> &mut NodeState<EchoTimer> {
        &mut self.state
    }
}
