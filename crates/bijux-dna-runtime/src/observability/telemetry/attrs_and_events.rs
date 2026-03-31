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
