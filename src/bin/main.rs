use std::io::{self, Write};

use dist::{
    broadcast::BroadcastState,
    id::IdState,
    messages::{Body, Message},
    nodes::Node,
};

struct Example {
    name: String,
    reddit: u32,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut node: Node<BroadcastState> = Node::init();
    node.run().await
}
