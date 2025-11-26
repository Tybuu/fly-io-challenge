use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum SeqPayload {
    Read { key: String },
    ReadOk { value: u32 },
    Write { key: String, value: u32 },
    WriteOk {},
    Cas { key: String, from: u32, to: u32 },
    CasOk {},
    Error { code: u32, text: String },
}
