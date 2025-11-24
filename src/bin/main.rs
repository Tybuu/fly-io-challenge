use std::io::{self, Write};

use dist::{
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
    let mut node: Node<IdState> = Node::init();
    node.run().await
}
