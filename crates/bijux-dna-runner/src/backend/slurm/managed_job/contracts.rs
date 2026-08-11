//! Typed identities and scheduler evidence for immutable external workers.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::super::{
    SlurmAccounting, SlurmDependencyCondition, SlurmJobObservation, SlurmSubmitRequest,
};

pub const MANAGED_SLURM_JOB_SPEC_SCHEMA_V1: &str = "bijux.runner.managed_slurm_job_spec.v1";
pub const PREPARED_MANAGED_SLURM_JOB_SCHEMA_V1: &str = "bijux.runner.prepared_managed_slurm_job.v1";
pub const MANAGED_SLURM_JOB_EVIDENCE_SCHEMA_V1: &str = "bijux.runner.managed_slurm_job_evidence.v1";
pub(super) const SUBMISSION_INTENT_SCHEMA_V1: &str =
    "bijux.runner.managed_slurm_submission_intent.v1";
pub(super) const SUBMISSION_FAILURE_SCHEMA_V1: &str =
    "bijux.runner.managed_slurm_submission_failure.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManagedSlurmJobResources {
    pub account: String,
    pub partition: String,
    pub cpus: u32,
    pub memory_mb: u64,
    pub walltime: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManagedSlurmJobSpec {
    pub schema_version: String,
    pub run_id: String,
    pub attempt_id: String,
    pub repository_path: PathBuf,
    pub repository_commit: String,
    pub worker_path: PathBuf,
    pub worker_sha256: String,
    pub attempt_root: PathBuf,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<String>,
    #[serde(default, skip_serializing_if = "SlurmDependencyCondition::is_after_ok")]
    pub dependency_condition: SlurmDependencyCondition,
    pub resources: ManagedSlurmJobResources,
    pub submitted_at_utc: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreparedManagedSlurmJob {
    pub schema_version: String,
    pub request: SlurmSubmitRequest,
    pub specification_sha256: String,
    pub worker_sha256: String,
    pub script_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManagedSlurmJobEvidence {
    pub schema_version: String,
    pub run_id: String,
    pub attempt_id: String,
    pub repository_commit: String,
    pub job_id: String,
    pub specification_sha256: String,
    pub worker_sha256: String,
    pub script_sha256: String,
    pub accounting: SlurmAccounting,
    pub stdout_sha256: String,
    pub stderr_sha256: String,
    pub collected_at_utc: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManagedSlurmJobCollection {
    pub observation: SlurmJobObservation,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence: Option<ManagedSlurmJobEvidence>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SubmissionIntent {
    pub schema_version: String,
    pub request: SlurmSubmitRequest,
    pub specification_sha256: String,
    pub worker_sha256: String,
    pub script_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SubmissionFailure {
    pub schema_version: String,
    pub run_id: String,
    pub submitted_at_utc: String,
    pub error: String,
}
