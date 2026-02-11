use bijux_dna_core::contract::MetricProvenanceV1;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum AttrValue {
    Str(String),
    Int(i64),
    Float(f64),
    Bool(bool),
}

pub type AttrMap = BTreeMap<String, AttrValue>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FailureCode {
    ToolFailed,
    MissingArtifact,
    InvalidParams,
    InvariantViolation,
    IoError,
    Timeout,
    ParseError,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TelemetryEventName {
    RunStarted,
    StageStart,
    ToolInvocation,
    StdoutSummary,
    StderrSummary,
    InvariantResult,
    ArtifactWritten,
    MetricsEmitted,
    StageEnd,
    RunFinished,
    RunFailed,
    MergeDecision,
    AdapterValidation,
    ContaminantAction,
    QualityGate,
    Error,
}

#[must_use]
pub fn attrs_from_json(value: &serde_json::Value) -> AttrMap {
    match value {
        serde_json::Value::Object(map) => map
            .iter()
            .filter_map(|(key, val)| AttrValue::from_json(val).map(|attr| (key.clone(), attr)))
            .collect(),
        _ => AttrMap::new(),
    }
}

impl AttrValue {
    fn from_json(value: &serde_json::Value) -> Option<Self> {
        match value {
            serde_json::Value::String(value) => Some(AttrValue::Str(value.clone())),
            serde_json::Value::Number(value) => {
                if let Some(int) = value.as_i64() {
                    Some(AttrValue::Int(int))
                } else {
                    value.as_f64().map(AttrValue::Float)
                }
            }
            serde_json::Value::Bool(value) => Some(AttrValue::Bool(*value)),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TelemetryEventV1 {
    pub schema_version: String,
    pub run_id: String,
    pub stage_id: String,
    pub tool_id: String,
    pub event_name: TelemetryEventName,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub duration_ms: Option<u64>,
    pub status: String,
    pub trace_id: String,
    pub span_id: String,
    #[serde(default)]
    pub attrs: AttrMap,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure_code: Option<FailureCode>,
}

const REDACT_TOKEN: &str = "[REDACTED]";
const SENSITIVE_KEY_PARTS: [&str; 7] = [
    "secret",
    "token",
    "password",
    "passwd",
    "apikey",
    "api_key",
    "authorization",
];

#[must_use]
pub fn redact_key(key: &str) -> bool {
    let lower = key.to_ascii_lowercase();
    SENSITIVE_KEY_PARTS
        .iter()
        .any(|needle| lower.contains(needle))
}

#[must_use]
pub fn redacted_attrs(attrs: &AttrMap) -> AttrMap {
    attrs
        .iter()
        .map(|(k, v)| {
            if redact_key(k) {
                (k.clone(), AttrValue::Str(REDACT_TOKEN.to_string()))
            } else {
                (k.clone(), v.clone())
            }
        })
        .collect()
}

/// Validate that an event list for a single stage has start/end coverage and artifact references.
#[must_use]
pub fn validate_stage_telemetry(events: &[TelemetryEventV1]) -> Vec<String> {
    let mut violations = Vec::new();
    let has_start = events
        .iter()
        .any(|event| matches!(event.event_name, TelemetryEventName::StageStart));
    let has_end = events
        .iter()
        .any(|event| matches!(event.event_name, TelemetryEventName::StageEnd));
    if !has_start {
        violations.push("missing stage_start event".to_string());
    }
    if !has_end {
        violations.push("missing stage_end event".to_string());
    }
    for event in events
        .iter()
        .filter(|event| matches!(event.event_name, TelemetryEventName::ArtifactWritten))
    {
        let has_ref =
            event.attrs.contains_key("artifact_id") || event.attrs.contains_key("artifact_path");
        if !has_ref {
            violations.push(format!(
                "artifact_written event missing artifact reference for stage {}",
                event.stage_id
            ));
        }
    }
    violations
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactsRowV1 {
    pub schema_version: String,
    pub run_id: String,
    pub stage_id: String,
    pub tool_id: String,
    pub tool_version: String,
    pub image_digest: Option<String>,
    pub trace_id: String,
    pub span_id: String,
    pub params_hash: String,
    pub input_hash: String,
    pub output_hashes: Vec<String>,
    pub runtime_s: f64,
    pub memory_mb: f64,
    pub exit_code: i32,
    pub bank_hashes: serde_json::Value,
    pub reads_in: Option<u64>,
    pub reads_out: Option<u64>,
    pub bases_in: Option<u64>,
    pub bases_out: Option<u64>,
    pub pairs_in: Option<u64>,
    pub pairs_out: Option<u64>,
    pub metrics: serde_json::Value,
    pub reports: serde_json::Value,
    pub artifacts: serde_json::Value,
}

impl FactsRowV1 {
    #[must_use]
    pub fn effective_metric_provenance(&self) -> MetricProvenanceV1 {
        MetricProvenanceV1 {
            run_id: self.run_id.clone(),
            stage_id: self.stage_id.clone(),
            tool_id: self.tool_id.clone(),
            tool_version: self.tool_version.clone(),
            params_hash: self.params_hash.clone(),
            input_artifact_hashes: vec![self.input_hash.clone()],
            manifest_hash: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunProvenanceV1 {
    pub schema_version: String,
    pub tool_image_digest: Option<String>,
    pub tool_version: String,
    pub params_hash: String,
    pub input_hashes: Vec<String>,
    pub reference_genome: Option<String>,
    pub pipeline_id: String,
    pub git_commit: String,
    pub build_profile: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plan_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RunContextV1 {
    Local,
    Hpc {
        site: String,
        scratch: String,
        slurm: bool,
    },
}
