use std::io::{self, Write};

use dist::{
    messages::{Body, Message},
    nodes::Node,
};

fn main() -> anyhow::Result<()> {
    let mut node = Node::init();
    node.run()
}
