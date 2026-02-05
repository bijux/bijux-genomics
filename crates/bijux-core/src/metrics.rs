use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::contract::ToolConstraints;
use crate::primitives::measure::ExecutionMetrics;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetricSet<T> {
    pub metrics_schema: String,
    pub version: i32,
    pub metrics: T,
}

impl<T> MetricSet<T> {
    #[must_use]
    pub fn new(metrics_schema: String, version: i32, metrics: T) -> Self {
        Self {
            metrics_schema,
            version,
            metrics,
        }
    }
}

pub type MetricEnvelope<T> = MetricSet<T>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StageMetricsV1<T> {
    pub schema_version: String,
    pub stage_id: String,
    pub stage_version: i32,
    pub tool_id: String,
    pub tool_version: String,
    pub context: MetricContextV1,
    pub execution: ExecutionMetrics,
    #[serde(default)]
    pub failure_class: Option<String>,
    #[serde(default)]
    pub failure_reason: Option<String>,
    pub metrics: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolInvocationV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub tool_id: String,
    pub tool_version: String,
    #[serde(default)]
    pub resolved_tool_version: Option<String>,
    pub image_digest: String,
    pub runner_kind: String,
    pub platform: String,
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
    pub resources: ToolConstraints,
    pub environment: BTreeMap<String, String>,
    pub input_hashes: Vec<String>,
    pub output_hashes: Vec<String>,
    #[serde(default)]
    pub executed_command: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdapterBankProvenanceV1 {
    pub bank_id: String,
    pub bank_version: String,
    pub bank_hash: String,
    pub presets_hash: String,
    pub preset: String,
    pub preset_hash: String,
    pub enabled_categories: Vec<String>,
    pub disabled_categories: Vec<String>,
    pub enable_adapters: Vec<String>,
    pub disable_adapters: Vec<String>,
    #[serde(default)]
    pub enabled_entries: Vec<BankEntryV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BankEntryV1 {
    pub id: String,
    pub sequence: String,
    pub rationale: String,
    pub source: String,
}
