//! Planned campaign outputs and preflight observations; these are not scheduler execution receipts.
use super::configuration::ResourceTemplate;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct CampaignPreflightReport {
    pub schema_version: &'static str,
    pub config_path: String,
    pub env_file_path: String,
    pub user_policy_path: String,
    pub user_policies_applied: bool,
    pub checks: Vec<CampaignCheck>,
    pub resolved_slurm: ResolvedSlurm,
    pub ok: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct CampaignDryRunReport {
    pub schema_version: &'static str,
    pub config_path: String,
    pub env_file_path: String,
    pub user_policy_path: String,
    pub user_policies_applied: bool,
    pub campaign_id: String,
    pub domain: String,
    pub layout: PlannedLayout,
    pub resolved_slurm: ResolvedSlurm,
    pub security: PlannedSecurity,
    pub planned_jobs: Vec<PlannedJob>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlannedLayout {
    pub corpora_root: String,
    pub databases_root: String,
    pub images_root: String,
    pub scratch_root: String,
    pub logs_root: String,
    pub encrypted_results_root: String,
    pub encrypted_code_root: String,
    pub appraiser_imports_root: String,
    pub baselines_root: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlannedSecurity {
    pub encryption_backend: String,
    pub encryption_recipients: Vec<String>,
    pub encryption_identity_files: Vec<String>,
    pub encrypt_operator_outputs: bool,
    pub redacted_env_keys: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlannedJob {
    pub job_id: String,
    pub job_name: String,
    pub stage: String,
    pub tool: String,
    pub sample: String,
    pub corpus_size_class: Option<String>,
    pub array_task: Option<u32>,
    pub depends_on: Vec<String>,
    pub resource_template: String,
    pub resources: ResourceTemplate,
    pub outputs: PlannedOutputs,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlannedOutputs {
    pub scratch_dir: String,
    pub log: String,
    pub out: String,
    pub err: String,
    pub results: String,
    pub code: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CampaignCheck {
    pub name: String,
    pub ok: bool,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResolvedSlurm {
    pub site_profile: String,
    pub account_redacted: String,
    pub project_redacted: String,
    pub partition: String,
    pub qos: String,
    pub default_resource_template: String,
    pub retry_attempts: u32,
    pub retry_backoff_seconds: u32,
    pub retry_on_exit_codes: Vec<i32>,
}

/// One local filesystem observation captured by the repository adapter.
#[derive(Debug, Clone)]
pub struct CampaignLayoutObservation {
    pub name: String,
    pub path: std::path::PathBuf,
    pub exists: bool,
    pub writable: bool,
}

/// Observations are supplied independently of pure preflight policy evaluation.
#[derive(Debug, Clone)]
pub struct CampaignPreflightObservations {
    pub env_file_private: Option<bool>,
    pub commands: std::collections::BTreeMap<String, bool>,
    pub layout: Vec<CampaignLayoutObservation>,
}

/// Borrowed declared inputs and observed facts for one preflight decision.
pub struct CampaignPreflightInputs<'a> {
    pub config: &'a super::configuration::CampaignConfig,
    pub config_path: &'a std::path::Path,
    pub env_file_path: &'a std::path::Path,
    pub user_policy_path: &'a std::path::Path,
    pub user_policies_applied: bool,
    pub resolved_slurm: ResolvedSlurm,
    pub observations: CampaignPreflightObservations,
}
