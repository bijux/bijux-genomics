//! Owner: bijux-dna-runner
//! Typed scheduler contracts for governed corpus materialization.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::super::{SlurmAccounting, SlurmJobObservation, SlurmSubmitRequest};

pub const CORPUS_MATERIALIZATION_SLURM_SPEC_SCHEMA_V1: &str =
    "bijux.runner.corpus_materialization_slurm_spec.v1";
pub const CORPUS_MATERIALIZATION_SLURM_SPEC_SCHEMA_V2: &str =
    "bijux.runner.corpus_materialization_slurm_spec.v2";
pub const CORPUS_MATERIALIZATION_PREPARED_JOB_SCHEMA_V1: &str =
    "bijux.runner.corpus_materialization_prepared_job.v1";
pub const CORPUS_MATERIALIZATION_EVIDENCE_SCHEMA_V1: &str =
    "bijux.runner.corpus_materialization_evidence.v1";
pub const CORPUS_MATERIALIZATION_EVIDENCE_SCHEMA_V2: &str =
    "bijux.runner.corpus_materialization_evidence.v2";
pub const CORPUS_MATERIALIZATION_EVIDENCE_CORRECTION_SCHEMA_V1: &str =
    "bijux.runner.corpus_materialization_evidence_correction.v1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CorpusMaterializationSlurmResources {
    pub account: String,
    pub partition: String,
    pub cpus: u32,
    pub memory_mb: u64,
    pub walltime: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CorpusMaterializationSlurmPlanRequest {
    pub repository_path: PathBuf,
    pub executable_path: PathBuf,
    pub plan_path: PathBuf,
    pub data_root: PathBuf,
    pub run_id: String,
    pub attempt_id: String,
    pub resources: CorpusMaterializationSlurmResources,
    pub maximum_parallelism: usize,
    pub retries: usize,
    pub timeout_seconds: u64,
    pub admitted_at_utc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CorpusMaterializationSlurmSpec {
    pub schema_version: String,
    pub run_id: String,
    pub attempt_id: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub corpus_id: String,
    pub portfolio_id: String,
    pub repository_path: PathBuf,
    pub repository_commit: String,
    pub executable_path: PathBuf,
    pub executable_sha256: String,
    #[serde(default)]
    pub command_arguments: Vec<String>,
    pub plan_path: PathBuf,
    pub plan_sha256: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub existing_catalog_sha256: Option<String>,
    pub data_root: PathBuf,
    pub evidence_root: PathBuf,
    pub resources: CorpusMaterializationSlurmResources,
    pub maximum_parallelism: usize,
    pub retries: usize,
    pub timeout_seconds: u64,
    pub admitted_at_utc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PreparedCorpusMaterializationSlurmJob {
    pub schema_version: String,
    pub request: SlurmSubmitRequest,
    pub specification_sha256: String,
    pub script_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CorpusMaterializationEvidenceStatus {
    Passed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CorpusMaterializationEvidence {
    pub schema_version: String,
    pub status: CorpusMaterializationEvidenceStatus,
    pub run_id: String,
    pub attempt_id: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub corpus_id: String,
    pub portfolio_id: String,
    pub repository_commit: String,
    pub job_id: String,
    pub specification_sha256: String,
    pub script_sha256: String,
    pub executable_sha256: String,
    pub plan_sha256: String,
    pub accounting: SlurmAccounting,
    pub stdout_sha256: String,
    pub stderr_sha256: String,
    pub publication_sha256: Option<String>,
    pub catalog_path: Option<PathBuf>,
    pub catalog_sha256: Option<String>,
    pub collected_at_utc: String,
    pub failure: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CorpusMaterializationCollection {
    pub observation: SlurmJobObservation,
    pub evidence: Option<CorpusMaterializationEvidence>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CorpusMaterializationEvidenceCorrectionRequest {
    pub evidence_path: PathBuf,
    pub output_path: PathBuf,
    pub observed_local_time: String,
    pub source_timezone: String,
    pub corrected_at_utc: String,
    pub source_commit: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CorpusMaterializationEvidenceCorrection {
    pub schema_version: String,
    pub run_id: String,
    pub attempt_id: String,
    pub job_id: String,
    pub evidence_path: PathBuf,
    pub evidence_sha256: String,
    pub field: String,
    pub recorded_value: String,
    pub observed_local_time: String,
    pub source_timezone: String,
    pub corrected_value: String,
    pub corrected_at_utc: String,
    pub source_commit: String,
    pub reason: String,
}
