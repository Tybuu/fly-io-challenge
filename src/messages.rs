use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    pub src: String,
    #[serde(rename = "dest")]
    pub dst: String,
    pub body: Body,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Body {
    #[serde(rename = "msg_id")]
    pub id: Option<usize>,
    #[serde(rename = "in_reply_to")]
    pub req_id: Option<usize>,
    #[serde(flatten)]
    pub payload: Payload,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Payload {
    Init {
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk {},
    Echo {
        echo: String,
    },
    EchoOk {
        echo: String,
    },
}
