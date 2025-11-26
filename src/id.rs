use serde::{Deserialize, Serialize};

use crate::{
    messages::Message,
    nodes::{MessageType, Node, NodeState},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum IdPayload {
    Generate {},
    GenerateOk { id: u64 },
}
pub struct IdNode {
    state: NodeState<IdTimer>,
    current_id: u64,
}

pub struct IdTimer {}

impl Node for IdNode {
    type PayloadType = IdPayload;
    type Timer = IdTimer;

    fn init() -> Self {
        Self {
            state: NodeState::init(),
            current_id: rand::random(),
        }
    }

    fn process_message(&mut self, message: MessageType<Self::PayloadType>) -> anyhow::Result<()> {
        if let MessageType::Defined(message) = message {
            match message.body.payload {
                IdPayload::Generate {} => {
                    self.current_id += 1;
                    let payload = IdPayload::GenerateOk {
                        id: self.current_id - 1,
                    };
                    self.write_message(&payload, message.body.id, message.src)?;
                }
                IdPayload::GenerateOk { .. } => unreachable!(),
            }
        }
        Ok(())
    }

    fn get_state(&self) -> &NodeState<IdTimer> {
        &self.state
    }

    fn get_state_mut(&mut self) -> &mut NodeState<IdTimer> {
        &mut self.state
    }

    fn handle_timer(&mut self, timer: Self::Timer) -> anyhow::Result<()> {
        Ok(())
    }
}
