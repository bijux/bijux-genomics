use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::contract::ContractVersion;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolProvenanceV1 {
    pub stage_id: String,
    pub tool_id: String,
    pub tool_version: String,
    pub image_digest: String,
    pub params_hash: String,
    pub parameters_json: serde_json::Value,
    pub input_hashes: Vec<String>,
    pub output_hashes: Vec<String>,
    #[serde(default)]
    pub banks: Option<serde_json::Value>,
    #[serde(default)]
    pub bank_assets: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScientificProvenanceV1 {
    pub schema_version: String,
    #[serde(default = "ContractVersion::v1")]
    pub contract_version: ContractVersion,
    pub pipeline_id: String,
    pub planner_version: String,
    pub tools: Vec<ToolProvenanceV1>,
    pub input_hashes: Vec<String>,
    pub reference_hashes: BTreeMap<String, String>,
}
