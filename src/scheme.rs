use serde::{Deserialize, Serialize};

// Constants Legacy
pub const DATA_PROTOCOL: &str = "ao";
pub const VARIANT: &str = "ao.TN.1";
pub const TYPE_MESSAGE: &str = "Message";
pub const TYPE_PROCESS: &str = "Process";
pub const SDK: &str = "rusty-ao";
pub const DEFAULT_MU: &str = "https://mu.ao-testnet.xyz";
pub const DEFAULT_CU: &str = "https://cu.ao-testnet.xyz";

pub const DEFAULT_MODULE: &str = "xT0ogTeagEGuySbKuUoo_NaWeeBv1fZ4MqgDdKVKY0U";
pub const DEFAULT_SQLITE_MODULE: &str = "sFNHeYzhHfP9vV9CPpqZMU-4Zzq_qKGKwlwMZozWi2Y";
pub const DEFAULT_SCHEDULER: &str = "_GQ33BkPtZrqxA84vM8Zk-N2aO0toNNu_C-l-rawrBA";

// Constants HyperBEAM
pub const HB_NODE_ENDPOINT: &str = "https://tee-1.forward.computer";

#[derive(Serialize, Deserialize, Debug)]
pub struct ResponseMu {
    pub id: String,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ResponseCu {
    pub messages: Vec<serde_json::Value>, // Placeholder for the actual type
    pub assignments: Vec<serde_json::Value>, // Placeholder for the actual type
    pub spawns: Vec<serde_json::Value>,   // Placeholder for the actual type
    pub output: serde_json::Value,        // Placeholder for the actual type
    pub gas_used: i64,
}

pub use bundlr_sdk::tags::Tag;