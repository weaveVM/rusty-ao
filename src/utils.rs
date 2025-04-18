use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use tiny_keccak::{Hasher, Keccak};
use std::fs;
use serde_json::from_str;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Node {
    pub node_url: String,
    pub base64: String,
    pub keccak256: String,
    pub address: String
}

pub fn encode_base64(input: &str) -> String {
    general_purpose::STANDARD.encode(input.as_bytes())
}

pub fn keccak256(input: &str) -> String {
    let mut hasher = Keccak::v256();
    let mut output = [0u8; 32];
    hasher.update(input.as_bytes());
    hasher.finalize(&mut output);
    hex::encode(output)
}

pub fn get_node(address: &str) -> Node {
    let nodes = fs::read_to_string("./compiled_node_list.json").unwrap();
    let nodes: Vec<Node> = serde_json::from_str(&nodes).unwrap();
    let node = nodes.iter().find(|node| node.address == address).cloned().unwrap_or_else(|| Node::default());
    node
}