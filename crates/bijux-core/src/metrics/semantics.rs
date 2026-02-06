//! Metric semantics and contextual metadata.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BankRefV1 {
    pub bank_id: String,
    pub bank_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetricContextV1 {
    pub tool_id: String,
    pub tool_version: String,
    pub image_digest: Option<String>,
    pub runner: String,
    pub platform: String,
    pub input_hash: String,
    pub params_hash: String,
    #[serde(default)]
    pub presets: std::collections::BTreeMap<String, String>,
    #[serde(default)]
    pub banks: std::collections::BTreeMap<String, BankRefV1>,
}
