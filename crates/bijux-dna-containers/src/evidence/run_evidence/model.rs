//! Persisted selected-run publication types.

use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const CONTAINER_RUN_EVIDENCE_INDEX_SCHEMA_V2: &str = "bijux.container.run_evidence_index.v2";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RunEvidenceIndex {
    pub schema_version: String,
    pub summary: RunEvidenceSummary,
    pub runs: Vec<RunEvidenceRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RunEvidenceSummary {
    pub run_count: usize,
    pub tool_count: usize,
    pub execution_status: BTreeMap<String, usize>,
    pub scientific_status: BTreeMap<String, usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RunEvidenceRecord {
    pub campaign: String,
    pub corpus: String,
    pub domain: String,
    pub evidence_lock_sha256: String,
    pub execution_status: AttemptExecutionStatus,
    pub pipeline_schema_version: String,
    pub repository_commit: String,
    pub run_id: String,
    pub sample: String,
    pub scientific_status: ScientificAcceptanceStatus,
    pub slurm_job_id: String,
    pub source_file_count: u64,
    pub source_result: String,
    pub stage: String,
    pub tool: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AttemptExecutionStatus {
    Passed,
    Failed,
    Blocked,
    Incomplete,
}

impl AttemptExecutionStatus {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Failed => "failed",
            Self::Blocked => "blocked",
            Self::Incomplete => "incomplete",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ScientificAcceptanceStatus {
    Passed,
    Failed,
    NotAssessed,
}

impl ScientificAcceptanceStatus {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Failed => "failed",
            Self::NotAssessed => "not_assessed",
        }
    }
}
