//! Owner: bijux-dna-runner
//! Typed scheduler contracts for container image campaigns.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::super::{SlurmAccounting, SlurmJobObservation, SlurmSubmitRequest};

pub const IMAGE_CAMPAIGN_SLURM_SPEC_SCHEMA_V1: &str = "bijux.runner.image_campaign_slurm_spec.v1";
pub const IMAGE_CAMPAIGN_PREPARED_JOB_SCHEMA_V1: &str =
    "bijux.runner.image_campaign_prepared_job.v1";
pub const IMAGE_CAMPAIGN_EVIDENCE_SCHEMA_V1: &str = "bijux.runner.image_campaign_evidence.v1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ImageCampaignSlurmResources {
    pub account: String,
    pub partition: String,
    pub cpus: u32,
    pub memory_mb: u64,
    pub walltime: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageCampaignSlurmPlanRequest {
    pub repository_path: PathBuf,
    pub executable_path: PathBuf,
    pub study_root: PathBuf,
    pub apptainer_path: PathBuf,
    pub attempt_id: String,
    pub selected_tools: Vec<String>,
    pub worker_count: usize,
    pub resources: ImageCampaignSlurmResources,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ImageCampaignSlurmSpec {
    pub schema_version: String,
    pub run_id: String,
    pub attempt_id: String,
    pub repository_path: PathBuf,
    pub repository_commit: String,
    pub executable_path: PathBuf,
    pub executable_sha256: String,
    pub study_root: PathBuf,
    pub apptainer_path: PathBuf,
    pub apptainer_sha256: String,
    pub attempt_root: PathBuf,
    pub selected_tools: Vec<String>,
    pub worker_count: usize,
    pub resources: ImageCampaignSlurmResources,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PreparedImageCampaignSlurmJob {
    pub schema_version: String,
    pub request: SlurmSubmitRequest,
    pub specification_sha256: String,
    pub script_sha256: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ImageCampaignEvidenceStatus {
    Passed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ImageCampaignEvidence {
    pub schema_version: String,
    pub status: ImageCampaignEvidenceStatus,
    pub run_id: String,
    pub attempt_id: String,
    pub repository_commit: String,
    pub selected_tools: Vec<String>,
    pub job_id: String,
    pub specification_sha256: String,
    pub script_sha256: String,
    pub executable_sha256: String,
    pub apptainer_sha256: String,
    pub accounting: SlurmAccounting,
    pub stdout_sha256: String,
    pub stderr_sha256: String,
    pub completion_sha256: Option<String>,
    pub campaign_execution_sha256: Option<String>,
    pub passed_tools: Option<usize>,
    pub failed_tools: Option<usize>,
    pub collected_at_utc: String,
    pub failure: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ImageCampaignCollection {
    pub observation: SlurmJobObservation,
    pub evidence: Option<ImageCampaignEvidence>,
}
