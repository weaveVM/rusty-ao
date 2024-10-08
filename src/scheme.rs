use serde::{Deserialize, Serialize};

// Constants
pub const DATA_PROTOCOL: &str = "ao";
pub const VARIANT: &str = "ao.TN.1";
pub const TYPE_MESSAGE: &str = "Message";
pub const TYPE_PROCESS: &str = "Process";
pub const SDK: &str = "goao";

pub const DEFAULT_MODULE: &str = "xT0ogTeagEGuySbKuUoo_NaWeeBv1fZ4MqgDdKVKY0U";
pub const DEFAULT_SQLITE_MODULE: &str = "sFNHeYzhHfP9vV9CPpqZMU-4Zzq_qKGKwlwMZozWi2Y";
pub const DEFAULT_SCHEDULER: &str = "_GQ33BkPtZrqxA84vM8Zk-N2aO0toNNu_C-l-rawrBA";

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ResponseMu {
    id: String,
    message: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ResponseCu {
    messages: Vec<serde_json::Value>, // Placeholder for the actual type
    assignments: Vec<serde_json::Value>, // Placeholder for the actual type
    spawns: Vec<serde_json::Value>,   // Placeholder for the actual type
    output: serde_json::Value,        // Placeholder for the actual type
    gas_used: i64,
}

pub struct Tag {
    pub name: String,
    pub value: String,
}
