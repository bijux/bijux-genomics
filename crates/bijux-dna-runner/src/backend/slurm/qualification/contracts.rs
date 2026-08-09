//! Owner: bijux-dna-runner
//! Typed scheduler contracts for governed container qualification.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::super::{SlurmAccounting, SlurmJobObservation, SlurmSubmitRequest};

pub const CONTAINER_QUALIFICATION_SLURM_SPEC_SCHEMA_V3: &str =
    "bijux.runner.container_qualification_slurm_spec.v3";
pub const CONTAINER_QUALIFICATION_SLURM_SPEC_SCHEMA_V4: &str =
    "bijux.runner.container_qualification_slurm_spec.v4";
pub const CONTAINER_QUALIFICATION_SLURM_SPEC_SCHEMA_V5: &str =
    "bijux.runner.container_qualification_slurm_spec.v5";
pub const CONTAINER_QUALIFICATION_PREPARED_JOB_SCHEMA_V1: &str =
    "bijux.runner.container_qualification_prepared_job.v1";
pub const CONTAINER_QUALIFICATION_EVIDENCE_SCHEMA_V1: &str =
    "bijux.runner.container_qualification_scheduler_evidence.v1";
pub const CONTAINER_QUALIFICATION_ACCOUNTING_SCHEMA_V1: &str =
    "bijux.runner.container_qualification_accounting.v1";
pub const CONTAINER_QUALIFICATION_FAILED_ATTEMPT_ARCHIVE_SCHEMA_V1: &str =
    "bijux.runner.container_qualification_failed_attempt_archive.v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainerQualificationPreparationRequest {
    pub attempt_id: String,
    pub hostname: String,
    pub governance_path: PathBuf,
    pub pipeline_path: PathBuf,
    pub credential_reference_path: PathBuf,
    pub executor_program_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoricalContainerQualificationReplayRequest {
    pub attempt_id: String,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub campaign_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attempt_id: Option<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ContainerQualificationExecution {
    pub passed: bool,
    pub evidence_path: PathBuf,
    pub evidence_sha256: String,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub executor_script_sha256: Option<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ContainerQualificationArchiveFile {
    pub path: String,
    pub size_bytes: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ContainerQualificationArchiveSources {
    pub submission: String,
    pub result: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ContainerQualificationFailedAttemptArchive {
    pub schema_version: String,
    pub run_id: String,
    pub tool_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attempt_id: Option<String>,
    pub slurm_job_id: String,
    pub archived_at_utc: String,
    pub source_paths: ContainerQualificationArchiveSources,
    pub scheduler_evidence_sha256: String,
    pub accounting_sha256: String,
    pub files: Vec<ContainerQualificationArchiveFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ArchivedContainerQualificationAttempt {
    pub path: PathBuf,
    pub archive: ContainerQualificationFailedAttemptArchive,
}
