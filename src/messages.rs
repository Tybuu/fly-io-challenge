use serde::{Deserialize, Serialize};

use std::fmt::Debug;
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message<P> {
    pub src: String,
    #[serde(rename = "dest")]
    pub dst: String,
    pub body: Body<P>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Body<P> {
    #[serde(rename = "msg_id")]
    pub id: Option<usize>,
    #[serde(rename = "in_reply_to")]
    pub req_id: Option<usize>,
    #[serde(flatten)]
    pub payload: P,
}
