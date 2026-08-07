//! Persisted selected-run publication types.

use std::collections::BTreeMap;
use std::path::PathBuf;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const CONTAINER_RUN_EVIDENCE_INDEX_SCHEMA_V2: &str = "bijux.container.run_evidence_index.v2";
pub const CONTAINER_RUN_EVIDENCE_LOCK_SCHEMA_V1: &str = "bijux.container.run_evidence_lock.v1";
pub const CONTAINER_RUN_ATTEMPT_LEDGER_SCHEMA_V1: &str = "bijux.container.run_attempt_ledger.v1";

#[derive(Debug, Clone)]
pub struct RunEvidenceImportRequest {
    pub result_root: PathBuf,
    pub output_root: PathBuf,
    pub minimum_run: Option<u64>,
    pub maximum_run: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FinalRunEvidencePlan {
    pub ledger: RunAttemptLedger,
    pub selected_run_count: usize,
    pub superseded_run_directories: Vec<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RunEvidenceLock {
    pub schema_version: String,
    pub run_id: String,
    pub tool: String,
    pub execution_status: AttemptExecutionStatus,
    pub scientific_status: ScientificAcceptanceStatus,
    pub source_result: String,
    pub repository_commit: String,
    pub slurm_job_id: String,
    pub image: serde_json::Value,
    pub failures: Vec<String>,
    pub source_files: Vec<RunEvidenceFileIdentity>,
    pub tracked_files: BTreeMap<String, RunEvidenceFileIdentity>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RunEvidenceFileIdentity {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    pub size_bytes: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RunAttemptLedger {
    pub schema_version: String,
    pub summary: RunAttemptLedgerSummary,
    pub attempts: Vec<RunAttemptLedgerRow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RunAttemptLedgerSummary {
    pub published_run_count: usize,
    pub superseded_attempt_count: usize,
    pub execution_status: BTreeMap<String, usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RunAttemptLedgerRow {
    pub run_id: String,
    pub tool: String,
    pub execution_status: AttemptExecutionStatus,
    pub repository_commit: String,
    pub slurm_job_id: String,
    pub source_result: String,
    pub source_file_count: usize,
    pub evidence_lock_sha256: String,
    pub failures: Vec<String>,
    pub superseded_by: String,
}

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
