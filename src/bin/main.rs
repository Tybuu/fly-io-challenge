use std::io::{self, Write};

use dist::{
    broadcast::BroadcastNode,
    counter::CounterNode,
    echo::EchoNode,
    id::IdNode,
    messages::{Body, Message},
    nodes::Node,
};

struct Example {
    name: String,
    reddit: u32,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut node = CounterNode::init();
    node.run().await
}
