use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum InvariantStatusV1 {
    Pass,
    Warn,
    Fail,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InvariantResultV1 {
    pub id: String,
    pub status: InvariantStatusV1,
    pub message: String,
    #[serde(default)]
    pub remediation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StageVerdictV1 {
    pub stage_id: String,
    pub verdict: InvariantStatusV1,
    pub reasons: Vec<String>,
    pub key_metrics: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InvariantSpecV1 {
    pub id: String,
    pub definition: String,
    pub threshold_provenance: String,
    pub severity: InvariantStatusV1,
    pub next_steps: String,
}
