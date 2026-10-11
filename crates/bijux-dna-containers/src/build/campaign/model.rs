//! Owner: image campaign requests and persisted plan, outcome and completion contracts.
use crate::build::CanonicalApptainerImagePublication;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
pub const IMAGE_CAMPAIGN_PLAN_SCHEMA_V1: &str = "bijux.container.image_campaign_plan.v1";
pub const IMAGE_CAMPAIGN_EXECUTION_SCHEMA_V1: &str = "bijux.container.image_campaign_execution.v1";
pub const IMAGE_CAMPAIGN_JOB_COMPLETION_SCHEMA_V1: &str = "bijux.image_campaign_job_completion.v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApptainerImageCampaignRequest {
    pub repository_root: PathBuf,
    pub study_root: PathBuf,
    pub apptainer_executable: PathBuf,
    pub worker_count: usize,
    /// Optional redistributable-image closure for an identity-addressed targeted campaign.
    pub selected_tools: Option<BTreeSet<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ImageCampaignPlan {
    pub schema_version: String,
    pub repository_commit: String,
    pub catalog_sha256: String,
    pub architecture: String,
    pub image_root: String,
    pub evidence_root: String,
    pub worker_count: usize,
    pub tools: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ImageCampaignToolStatus {
    Passed,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ImageCampaignToolOutcome {
    pub status: ImageCampaignToolStatus,
    pub publication: Option<CanonicalApptainerImagePublication>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ImageCampaignExecution {
    pub schema_version: String,
    pub repository_commit: String,
    pub catalog_sha256: String,
    pub architecture: String,
    pub tools: BTreeMap<String, ImageCampaignToolOutcome>,
    pub passed: usize,
    pub failed: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImageCampaignJobCompletion {
    pub schema_version: String,
    pub attempt_id: String,
    pub repository_commit: String,
    pub slurm_job_id: String,
    pub completed_at: String,
    pub campaign_execution: String,
    pub campaign_execution_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageCampaignAttemptRequest {
    pub campaign: ApptainerImageCampaignRequest,
    pub attempt_root: PathBuf,
    pub attempt_id: String,
    pub slurm_job_id: String,
    pub completed_at: String,
}
