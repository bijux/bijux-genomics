use serde::{Deserialize, Serialize};

use bijux_core::contract::ToolConstraints;
use bijux_core::metrics::{AdapterBankProvenanceV1, MetricContextV1};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StageObservabilityContractV1 {
    pub required_artifacts: Vec<String>,
    pub required_metadata_fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StageObservabilityContextV1 {
    pub stage_id: String,
    pub stage_version: i32,
    pub tool_id: String,
    pub tool_version: String,
    pub input_fingerprint: String,
    pub parameters_fingerprint: String,
    pub parameters_json: serde_json::Value,
    pub parameters_json_normalized: serde_json::Value,
    pub metric_context: MetricContextV1,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectiveConfigV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub stage_version: i32,
    pub tool_id: String,
    pub tool_version: String,
    pub image_digest: Option<String>,
    pub runner: String,
    pub platform: String,
    pub resources: ToolConstraints,
    pub parameters_json: serde_json::Value,
    pub parameters_json_normalized: serde_json::Value,
    #[serde(default)]
    pub effective_params_json: serde_json::Value,
    #[serde(default)]
    pub effective_params_json_normalized: serde_json::Value,
    #[serde(default)]
    pub adapter_bank: Option<AdapterBankProvenanceV1>,
    #[serde(default)]
    pub banks: Option<serde_json::Value>,
    #[serde(default)]
    pub bank_assets: Option<serde_json::Value>,
}
