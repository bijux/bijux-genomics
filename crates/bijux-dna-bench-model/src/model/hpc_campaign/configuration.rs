//! Campaign configuration, resource declarations and policy inputs.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;
pub const CAMPAIGN_SCHEMA_VERSION: &str = "bijux.hpc.campaign.v1";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CampaignConfig {
    #[serde(default = "default_campaign_schema_version")]
    pub schema_version: String,
    pub campaign: CampaignMeta,
    pub layout: CampaignLayout,
    #[serde(default)]
    pub slurm: CampaignSlurm,
    #[serde(default)]
    pub output_templates: OutputTemplates,
    #[serde(default)]
    pub resources: ResourceTemplates,
    #[serde(default)]
    pub security: CampaignSecurity,
    #[serde(default)]
    pub site_profiles: BTreeMap<String, SiteProfile>,
    #[serde(default)]
    pub jobs: Vec<CampaignJob>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CampaignMeta {
    pub id: String,
    pub domain: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CampaignLayout {
    pub corpora_root: PathBuf,
    pub databases_root: PathBuf,
    pub images_root: PathBuf,
    pub scratch_root: PathBuf,
    pub logs_root: PathBuf,
    pub encrypted_results_root: PathBuf,
    pub encrypted_code_root: PathBuf,
    pub appraiser_imports_root: PathBuf,
    pub baselines_root: PathBuf,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct CampaignSlurm {
    pub site_profile: Option<String>,
    pub account: Option<String>,
    pub project: Option<String>,
    pub partition: Option<String>,
    pub qos: Option<String>,
    pub mail_user: Option<String>,
    pub default_resource_template: Option<String>,
    pub retry_attempts: Option<u32>,
    pub retry_backoff_seconds: Option<u32>,
    #[serde(default)]
    pub retry_on_exit_codes: Vec<i32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CampaignJob {
    #[serde(default)]
    pub name: Option<String>,
    pub stage: String,
    pub tool: String,
    pub sample: String,
    #[serde(default)]
    pub corpus_size_class: Option<String>,
    #[serde(default)]
    pub depends_on: Vec<String>,
    #[serde(default)]
    pub array_task: Option<u32>,
    #[serde(default)]
    pub resource_template: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OutputTemplates {
    #[serde(default = "default_log_template")]
    pub log: String,
    #[serde(default = "default_out_template")]
    pub out: String,
    #[serde(default = "default_err_template")]
    pub err: String,
    #[serde(default = "default_results_template")]
    pub results: String,
    #[serde(default = "default_code_template")]
    pub code: String,
}

impl Default for OutputTemplates {
    fn default() -> Self {
        Self {
            log: default_log_template(),
            out: default_out_template(),
            err: default_err_template(),
            results: default_results_template(),
            code: default_code_template(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ResourceTemplate {
    pub cpus: u32,
    pub mem_gb: u32,
    pub walltime: String,
    pub scratch_gb: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ResourceTemplates {
    #[serde(default = "default_resource_template_name")]
    pub default: String,
    #[serde(default = "default_resource_template_map")]
    pub templates: BTreeMap<String, ResourceTemplate>,
    #[serde(default)]
    pub stage_defaults: BTreeMap<String, String>,
    #[serde(default)]
    pub corpus_size_scaling: BTreeMap<String, CorpusSizeScalePolicy>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CorpusSizeScalePolicy {
    #[serde(default = "default_scale_percent")]
    pub cpu_percent: u32,
    #[serde(default = "default_scale_percent")]
    pub mem_percent: u32,
    #[serde(default = "default_scale_percent")]
    pub walltime_percent: u32,
    #[serde(default = "default_scale_percent")]
    pub scratch_percent: u32,
}

impl Default for ResourceTemplates {
    fn default() -> Self {
        Self {
            default: default_resource_template_name(),
            templates: default_resource_template_map(),
            stage_defaults: BTreeMap::new(),
            corpus_size_scaling: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CampaignSecurity {
    #[serde(default)]
    pub env_file: Option<PathBuf>,
    #[serde(default)]
    pub redacted_env_keys: Vec<String>,
    #[serde(default = "default_encryption_backend")]
    pub encryption_backend: String,
    #[serde(default)]
    pub encryption_recipients: Vec<String>,
    #[serde(default)]
    pub encryption_identity_files: Vec<PathBuf>,
    #[serde(default)]
    pub encrypt_operator_outputs: bool,
}

impl Default for CampaignSecurity {
    fn default() -> Self {
        Self {
            env_file: None,
            redacted_env_keys: Vec::new(),
            encryption_backend: default_encryption_backend(),
            encryption_recipients: Vec::new(),
            encryption_identity_files: Vec::new(),
            encrypt_operator_outputs: false,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct SiteProfile {
    pub partition: Option<String>,
    pub qos: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct CampaignPolicies {
    #[serde(default)]
    pub layout: Option<LayoutPolicies>,
    #[serde(default)]
    pub slurm: Option<SlurmPolicies>,
    #[serde(default)]
    pub resources: Option<ResourcePolicies>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct LayoutPolicies {
    pub corpora_root: Option<PathBuf>,
    pub databases_root: Option<PathBuf>,
    pub images_root: Option<PathBuf>,
    pub scratch_root: Option<PathBuf>,
    pub logs_root: Option<PathBuf>,
    pub encrypted_results_root: Option<PathBuf>,
    pub encrypted_code_root: Option<PathBuf>,
    pub appraiser_imports_root: Option<PathBuf>,
    pub baselines_root: Option<PathBuf>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct SlurmPolicies {
    pub site_profile: Option<String>,
    pub account: Option<String>,
    pub project: Option<String>,
    pub partition: Option<String>,
    pub qos: Option<String>,
    pub default_resource_template: Option<String>,
    pub retry_attempts: Option<u32>,
    pub retry_backoff_seconds: Option<u32>,
    pub retry_on_exit_codes: Option<Vec<i32>>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ResourcePolicies {
    pub default: Option<String>,
    #[serde(default)]
    pub templates: BTreeMap<String, ResourceTemplate>,
}

fn default_campaign_schema_version() -> String {
    CAMPAIGN_SCHEMA_VERSION.to_string()
}

fn default_encryption_backend() -> String {
    "mock-envelope-v1".to_string()
}

/// Whether the named campaign encryption backend is supported.
#[must_use]
pub fn is_supported_encryption_backend(backend: &str) -> bool {
    matches!(backend, "mock-envelope-v1" | "age-cli")
}

fn default_resource_template_name() -> String {
    "standard".to_string()
}

fn default_scale_percent() -> u32 {
    100
}

fn default_resource_template_map() -> BTreeMap<String, ResourceTemplate> {
    BTreeMap::from([(
        "standard".to_string(),
        ResourceTemplate {
            cpus: 16,
            mem_gb: 64,
            walltime: "04:00:00".to_string(),
            scratch_gb: 128,
        },
    )])
}

fn default_log_template() -> String {
    "{campaign}/{domain}/{stage}/{tool}/{sample}/{job_id}-{timestamp}.log".to_string()
}

fn default_out_template() -> String {
    "{campaign}/{domain}/{stage}/{tool}/{sample}/{job_id}-{timestamp}.out".to_string()
}

fn default_err_template() -> String {
    "{campaign}/{domain}/{stage}/{tool}/{sample}/{job_id}-{timestamp}.err".to_string()
}

fn default_results_template() -> String {
    "{campaign}/{domain}/{stage}/{tool}/{sample}/{job_id}-{timestamp}.results".to_string()
}

fn default_code_template() -> String {
    "{campaign}/{domain}/{stage}/{tool}/{sample}/{job_id}-{timestamp}.code".to_string()
}
