//! Owner: bijux-dna-bench-model
//! Persisted benchmark suite run manifest and evidence rows.

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{de::Error as _, Deserialize, Deserializer, Serialize};

use crate::contract::BENCHMARK_SUITE_RUN_MANIFEST_SCHEMA_V2;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SuiteRunManifest {
    #[serde(deserialize_with = "deserialize_suite_run_manifest_schema_version")]
    #[schemars(schema_with = "suite_run_manifest_schema_version")]
    pub schema_version: String,
    pub suite_id: String,
    pub run_id: String,
    pub run_context: String,
    pub corpus: String,
    pub species_id: Option<String>,
    pub fairness: FairnessContract,
    pub cold_vs_warm: ColdWarmContract,
    pub decision_trace: Vec<DecisionTraceRow>,
    pub tool_invocations: Vec<ToolInvocationRow>,
    pub metrics_artifacts: Vec<MetricsArtifactRow>,
    pub postconditions: Vec<PostconditionRow>,
    pub run_records: Vec<RunRecordRow>,
    pub comparability_hash: String,
    pub environment: EnvironmentSnapshot,
    pub scientific_defaults: ScientificDefaultsStatus,
    pub telemetry_path: String,
    pub reproducibility_bundle: String,
    pub created_at_unix: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct FairnessContract {
    pub threads: u32,
    pub mem_gb: u32,
    pub tmp_policy: String,
    pub tmp_root: String,
    pub deviations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ColdWarmContract {
    pub cold_runs: u32,
    pub warm_runs: u32,
    pub cold_run_records_image_cost: bool,
    pub warm_runs_exclude_image_cost: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DecisionTraceRow {
    pub stage: String,
    pub candidates: Vec<String>,
    pub selected_tool: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ToolInvocationRow {
    pub stage: String,
    pub tool: String,
    pub mode: String,
    pub run_index: u32,
    pub tool_version: String,
    pub image_digest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct MetricsArtifactRow {
    pub stage: String,
    pub tool: String,
    pub mode: String,
    pub run_index: u32,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PostconditionRow {
    pub stage: String,
    pub tool: String,
    pub mode: String,
    pub run_index: u32,
    pub ok: bool,
    pub checks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RunRecordRow {
    pub stage: String,
    pub tool: String,
    pub mode: String,
    pub run_index: u32,
    pub runtime_s: f64,
    pub memory_mb: f64,
    pub read_retention: f64,
    pub length_shift: f64,
    pub delta_metrics: Option<serde_json::Value>,
    pub valid: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct EnvironmentSnapshot {
    pub apptainer_version: String,
    pub kernel: String,
    pub site_lock: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ScientificDefaultsStatus {
    pub doc_path: String,
    pub reference_defaults_ok: bool,
    pub checked_rows: usize,
}

#[must_use]
pub fn benchmark_suite_run_manifest_schema() -> Schema {
    schemars::schema_for!(SuiteRunManifest)
}

fn suite_run_manifest_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": BENCHMARK_SUITE_RUN_MANIFEST_SCHEMA_V2,
    })
}

fn deserialize_suite_run_manifest_schema_version<'de, D>(
    deserializer: D,
) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    if value == BENCHMARK_SUITE_RUN_MANIFEST_SCHEMA_V2 {
        Ok(value)
    } else {
        Err(D::Error::custom(format!("unsupported benchmark suite run manifest: {value}")))
    }
}
