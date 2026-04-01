use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StratificationRequirement {
    pub key: String,
    pub required_values: Vec<String>,
}
