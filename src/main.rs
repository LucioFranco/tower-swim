use tokio;
use tower_direct_service::DirectService;
use tower_swim::{Node, Packet};

fn main() {
    let mut node = Node::new();

    node.call(Packet::Ping(0));

    tokio::run(node);
}
