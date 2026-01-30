use serde::{Deserialize, Serialize};

use crate::{metrics::AdapterBankProvenanceV1, ToolConstraints};

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
    pub input_hash: String,
    pub params_hash: String,
    pub parameters_json: serde_json::Value,
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
    pub resources: ToolConstraints,
    pub parameters_json: serde_json::Value,
    #[serde(default)]
    pub adapter_bank: Option<AdapterBankProvenanceV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StageReportV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub stage_version: i32,
    pub tool_id: String,
    pub tool_version: String,
    pub metrics_path: String,
    pub effective_config_path: String,
    pub facts_row_id: Option<String>,
    pub summary: serde_json::Value,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub outputs: Vec<String>,
    pub subreports: Vec<String>,
    pub log_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RetentionReportV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub tool_id: String,
    pub tool_version: String,
    pub boundary: String,
    pub numerator: serde_json::Value,
    pub denominator: serde_json::Value,
    pub scope: String,
    pub condition: serde_json::Value,
    pub parameters_json: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrimReportV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub tool_id: String,
    pub reads_in: u64,
    pub reads_out: u64,
    pub bases_in: u64,
    pub bases_out: u64,
    pub bases_trimmed: u64,
    pub per_adapter_counts: std::collections::BTreeMap<String, u64>,
    #[serde(default)]
    pub adapter_preset: Option<String>,
    #[serde(default)]
    pub adapter_bank_id: Option<String>,
    #[serde(default)]
    pub adapter_bank_hash: Option<String>,
    #[serde(default)]
    pub adapter_overrides: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValidateReportV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub tool_id: String,
    pub reads_total: u64,
    pub reads_valid: u64,
    pub reads_invalid: u64,
    pub integrity_ok: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MergeReportV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub tool_id: String,
    pub reads_r1: u64,
    pub reads_r2: u64,
    pub reads_merged: u64,
    pub reads_unmerged: u64,
    pub merge_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TelemetryEventV1 {
    pub run_id: String,
    pub stage_id: String,
    pub tool_id: String,
    pub event_name: String,
    pub timestamp: String,
    pub duration_ms: Option<u64>,
    pub status: String,
    pub trace_id: String,
    pub span_id: String,
    pub attrs: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FactsRowV1 {
    pub schema_version: String,
    pub run_id: String,
    pub stage_id: String,
    pub tool_id: String,
    pub params_hash: String,
    pub input_hash: String,
    pub output_hashes: Vec<String>,
    pub runtime_s: f64,
    pub memory_mb: f64,
    pub exit_code: i32,
    pub metrics: serde_json::Value,
    pub artifacts: serde_json::Value,
}

pub fn canonicalize_json_value(value: &serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            let mut ordered = serde_json::Map::new();
            for key in keys {
                let val = map.get(key).unwrap_or(&serde_json::Value::Null);
                ordered.insert(key.clone(), canonicalize_json_value(val));
            }
            serde_json::Value::Object(ordered)
        }
        serde_json::Value::Array(items) => {
            serde_json::Value::Array(items.iter().map(canonicalize_json_value).collect())
        }
        _ => value.clone(),
    }
}

#[must_use]
pub fn parameters_json_canonicalization(value: &serde_json::Value) -> serde_json::Value {
    fn normalize_numbers(value: &serde_json::Value) -> serde_json::Value {
        match value {
            serde_json::Value::Number(num) => {
                if let Some(f) = num.as_f64() {
                    serde_json::Number::from_f64(f).map_or_else(
                        || serde_json::Value::Number(num.clone()),
                        serde_json::Value::Number,
                    )
                } else {
                    serde_json::Value::Number(num.clone())
                }
            }
            serde_json::Value::Array(items) => {
                serde_json::Value::Array(items.iter().map(normalize_numbers).collect())
            }
            serde_json::Value::Object(map) => {
                let mut ordered = serde_json::Map::new();
                for (key, val) in map {
                    ordered.insert(key.clone(), normalize_numbers(val));
                }
                serde_json::Value::Object(ordered)
            }
            _ => value.clone(),
        }
    }

    let canonical = canonicalize_json_value(value);
    normalize_numbers(&canonical)
}
