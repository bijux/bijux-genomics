//! Owner: bijux-dna-runner
//! Typed scheduler contracts for governed container qualification.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::super::{SlurmAccounting, SlurmJobObservation, SlurmSubmitRequest};

pub const CONTAINER_QUALIFICATION_SLURM_SPEC_SCHEMA_V1: &str =
    "bijux.runner.container_qualification_slurm_spec.v1";
pub const CONTAINER_QUALIFICATION_SLURM_SPEC_SCHEMA_V2: &str =
    "bijux.runner.container_qualification_slurm_spec.v2";
pub const CONTAINER_QUALIFICATION_PREPARED_JOB_SCHEMA_V1: &str =
    "bijux.runner.container_qualification_prepared_job.v1";
pub const CONTAINER_QUALIFICATION_EVIDENCE_SCHEMA_V1: &str =
    "bijux.runner.container_qualification_scheduler_evidence.v1";
pub const CONTAINER_QUALIFICATION_ACCOUNTING_SCHEMA_V1: &str =
    "bijux.runner.container_qualification_accounting.v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainerQualificationPreparationRequest {
    pub hostname: String,
    pub governance_path: PathBuf,
    pub pipeline_path: PathBuf,
    pub credential_reference_path: PathBuf,
    pub executor_program_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoricalContainerQualificationReplayRequest {
    pub hostname: String,
    pub governance_path: PathBuf,
    pub source_pipeline_path: PathBuf,
    pub replay_pipeline_path: PathBuf,
    pub credential_reference_path: PathBuf,
    pub executor_program_path: PathBuf,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ContainerQualificationSubmissionKind {
    #[default]
    Current,
    HistoricalReplay,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct HistoricalContainerQualificationReplaySource {
    pub pipeline_path: PathBuf,
    pub pipeline_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ContainerQualificationSlurmResources {
    pub account: String,
    pub partition: String,
    pub cpus: u64,
    pub memory_mb: u64,
    pub walltime: String,
    pub scratch_mb: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ContainerQualificationSlurmSpec {
    pub schema_version: String,
    #[serde(default)]
    pub submission_kind: ContainerQualificationSubmissionKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub historical_replay_source: Option<HistoricalContainerQualificationReplaySource>,
    pub run_id: String,
    pub tool_id: String,
    pub job_name: String,
    pub hostname: String,
    pub repository_path: PathBuf,
    pub repository_commit: String,
    pub governance_path: PathBuf,
    pub governance_sha256: String,
    pub pipeline_path: PathBuf,
    pub pipeline_sha256: String,
    pub credential_reference_path: PathBuf,
    pub credential_reference_sha256: String,
    pub executor_program_path: PathBuf,
    pub executor_program_sha256: String,
    pub executor_script_path: PathBuf,
    pub executor_script_sha256: String,
    pub result_root: PathBuf,
    pub evidence_root: PathBuf,
    pub resources: ContainerQualificationSlurmResources,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PreparedContainerQualificationSlurmJob {
    pub schema_version: String,
    pub request: SlurmSubmitRequest,
    pub specification_sha256: String,
    pub script_sha256: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ContainerQualificationSchedulerStatus {
    Passed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ContainerQualificationSchedulerEvidence {
    pub schema_version: String,
    pub status: ContainerQualificationSchedulerStatus,
    pub run_id: String,
    pub tool_id: String,
    pub repository_commit: String,
    pub job_id: String,
    pub specification_sha256: String,
    pub script_sha256: String,
    pub governance_sha256: String,
    pub pipeline_sha256: String,
    pub credential_reference_sha256: String,
    pub executor_program_sha256: String,
    pub executor_script_sha256: String,
    pub accounting: SlurmAccounting,
    pub stdout_sha256: String,
    pub stderr_sha256: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution_evidence_sha256: Option<String>,
    pub collected_at_utc: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ContainerQualificationAccountingRecord {
    pub schema_version: String,
    pub run_id: String,
    pub tool_id: String,
    pub slurm_job_id: String,
    pub accounting: SlurmAccounting,
    pub specification_sha256: String,
    pub script_sha256: String,
    pub governance_sha256: String,
    pub pipeline_sha256: String,
    pub credential_reference_sha256: String,
    pub collected_at_utc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ContainerQualificationCollection {
    pub observation: SlurmJobObservation,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence: Option<ContainerQualificationSchedulerEvidence>,
}
