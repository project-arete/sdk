use super::Stats;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Cache {
    pub version: String,
    pub stats: Stats,
    pub keys: HashMap<String, Value>,
}
