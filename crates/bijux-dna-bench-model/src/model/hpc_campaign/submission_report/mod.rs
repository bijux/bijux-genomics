//! Versioned scheduler submission receipts and frontend-independent scope settings.
use serde::{Deserialize, Serialize};
mod validation;
pub use validation::{validate_hpc_scheduler_receipt, validate_hpc_submission_report};

pub const HPC_SLURM_SUBMISSION_SCHEMA_VERSION: &str = "bijux.hpc.slurm.submission.v1";

// Decoding returns the admitted constant, so this field never borrows from input bytes.
// The alias prevents serde from inferring an input lifetime from a reference field.
type StaticSchemaVersion = &'static str;

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SlurmSubmissionReport {
    #[serde(deserialize_with = "validation::deserialize_version")]
    #[schemars(schema_with = "validation::version_schema")]
    pub schema_version: StaticSchemaVersion,
    #[serde(deserialize_with = "validation::deserialize_mode")]
    #[schemars(schema_with = "validation::mode_schema")]
    pub mode: String,
    pub campaign_id: String,
    pub domain: String,
    pub submitted_at: String,
    pub jobs: Vec<SubmittedJob>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SubmittedJob {
    pub job_name: String,
    pub stage: String,
    pub tool: String,
    pub sample: String,
    pub array_task: Option<u32>,
    pub planned_job_id: String,
    pub scheduler_job_id: String,
    pub dependency_scheduler_ids: Vec<String>,
    pub script_path: String,
    pub log_path: String,
    pub out_path: String,
    pub err_path: String,
    pub results_path: String,
    pub code_path: String,
}

#[derive(Debug, Clone)]
pub struct HpcSubmissionSettings {
    pub mode: HpcSubmissionMode,
    pub subset: crate::HpcSubmissionSelector,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HpcSubmissionMode {
    Mock,
    Real,
}
