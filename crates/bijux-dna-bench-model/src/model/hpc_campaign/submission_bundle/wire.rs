//! Closed JSON wire contracts emitted at HPC submission, before execution results exist.
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub(crate) enum ResultsBundleVersion {
    #[serde(rename = "bijux.hpc.results_bundle.v1")]
    V1,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub(crate) enum CodeBundleVersion {
    #[serde(rename = "bijux.hpc.code_bundle.v1")]
    V1,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct BundleCampaign {
    pub(crate) id: String,
    pub(crate) domain: String,
    pub(crate) config_path: String,
    pub(crate) env_file_path: String,
    pub(crate) user_policy_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct BundleCodeCampaign {
    pub(crate) id: String,
    pub(crate) domain: String,
    pub(crate) config_path: String,
    pub(crate) env_file_path: String,
    pub(crate) user_policy_path: String,
    pub(crate) user_policies_applied: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct BundleJob {
    pub(crate) planned_job_id: String,
    pub(crate) scheduler_job_id: String,
    pub(crate) submitted_at: String,
    pub(crate) job_name: String,
    pub(crate) stage: String,
    pub(crate) tool: String,
    pub(crate) sample: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct BundleResultsJob {
    pub(crate) planned_job_id: String,
    pub(crate) scheduler_job_id: String,
    pub(crate) submitted_at: String,
    pub(crate) job_name: String,
    pub(crate) stage: String,
    pub(crate) tool: String,
    pub(crate) sample: String,
    pub(crate) resource_template: String,
    pub(crate) resources: BundleResources,
    pub(crate) depends_on: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct BundleResources {
    pub(crate) cpus: u32,
    pub(crate) mem_gb: u32,
    pub(crate) walltime: String,
    pub(crate) scratch_gb: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct BundleSlurm {
    pub(crate) site_profile: String,
    pub(crate) account_redacted: String,
    pub(crate) project_redacted: String,
    pub(crate) partition: String,
    pub(crate) qos: String,
    pub(crate) default_resource_template: String,
    pub(crate) retry_attempts: u32,
    pub(crate) retry_backoff_seconds: u32,
    pub(crate) retry_on_exit_codes: Vec<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct BundleOutputs {
    pub(crate) scratch_dir: String,
    pub(crate) log: String,
    pub(crate) out: String,
    pub(crate) err: String,
    pub(crate) results: String,
    pub(crate) code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct BundleMetrics {
    pub(crate) status: String,
    pub(crate) submission_mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct BundleArtifact {
    pub(crate) kind: String,
    pub(crate) path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct BundleArtifacts {
    pub(crate) inventory: Vec<BundleArtifact>,
    pub(crate) encrypted_targets: Vec<BundleArtifact>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct BundleReport {
    pub(crate) kind: String,
    pub(crate) summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct BundleTraces {
    pub(crate) script_path: String,
    pub(crate) scheduler_job_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct BundleInventories {
    pub(crate) encryption_backend: String,
    pub(crate) recipient_fingerprints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct BundleAppraiser {
    pub(crate) name: String,
    pub(crate) status: String,
    pub(crate) note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct BundleEffectiveSettings {
    pub(crate) slurm: BundleSlurm,
    pub(crate) resources: BundleResources,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct BundleConfigReferences {
    pub(crate) campaign_config: String,
    pub(crate) env_file: String,
    pub(crate) user_policy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct BundleRepositoryState {
    #[serde(rename = "git_head")]
    pub(crate) head: Option<String>,
    #[serde(rename = "git_branch")]
    pub(crate) branch: Option<String>,
    #[serde(rename = "git_status_porcelain")]
    pub(crate) status_porcelain: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct BundleDvcState {
    pub(crate) available: bool,
    pub(crate) status_hint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct BundleCodeFreeze {
    pub(crate) slurm_script: String,
    pub(crate) effective_settings: BundleEffectiveSettings,
    pub(crate) config_references: BundleConfigReferences,
    pub(crate) repository_state: BundleRepositoryState,
    pub(crate) dvc_state: BundleDvcState,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct BundleLocks {
    #[serde(rename = "corpus_lock")]
    pub(crate) corpus: String,
    #[serde(rename = "database_lock")]
    pub(crate) database: String,
    #[serde(rename = "image_lock")]
    pub(crate) image: String,
    #[serde(rename = "tool_lock")]
    pub(crate) tool: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct BundlePlan {
    pub(crate) depends_on: Vec<String>,
    pub(crate) outputs: BundleOutputs,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct HpcResultsSubmissionBundle {
    pub(crate) schema_version: ResultsBundleVersion,
    pub(crate) campaign: BundleCampaign,
    pub(crate) job: BundleResultsJob,
    pub(crate) metrics: BundleMetrics,
    pub(crate) artifacts: BundleArtifacts,
    pub(crate) reports: Vec<BundleReport>,
    pub(crate) traces: BundleTraces,
    pub(crate) inventories: BundleInventories,
    pub(crate) appraiser_outputs: Vec<BundleAppraiser>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct HpcCodeSubmissionBundle {
    pub(crate) schema_version: CodeBundleVersion,
    pub(crate) campaign: BundleCodeCampaign,
    pub(crate) job: BundleJob,
    pub(crate) code_freeze: BundleCodeFreeze,
    pub(crate) locks: BundleLocks,
    pub(crate) plan: BundlePlan,
}
