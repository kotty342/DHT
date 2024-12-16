use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::net::SocketAddr;

#[derive(Clone)]
struct Node {
    id: String,
    address: SocketAddr,
}

struct DHT {
    nodes: Arc<Mutex<HashMap<String, Node>>>,
}

impl DHT {
    fn new() -> Self {
        DHT {
            nodes: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn add_node(&self, id: String, address: SocketAddr) {
        let mut nodes = self.nodes.lock().unwrap();
        nodes.insert(id.clone(), Node { id, address });
    }

    fn get_node(&self, id: &str) -> Option<Node> {
        let nodes = self.nodes.lock().unwrap();
        nodes.get(id).cloned()
    }
}

fn main() {
    let dht = DHT::new();
    let node_id = "node1".to_string();
    let node_address: SocketAddr = "127.0.0.1:8080".parse().unwrap();

    dht.add_node(node_id.clone(), node_address);

    if let Some(node) = dht.get_node(&node_id) {
        println!("Found node: {} at {}", node.id, node.address);
    } else {
        println!("Node not found");
    }
}
