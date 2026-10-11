//! Inputs supplied by campaign planning and transport observations, independent of CLI types.
use crate::{CampaignDryRunReport, PlannedJob};

/// One already planned job and its submission receipt identity.
pub struct HpcBundleSubmission<'a> {
    pub campaign: &'a CampaignDryRunReport,
    pub job: &'a PlannedJob,
    pub scheduler_job_id: &'a str,
    pub submitted_at: &'a str,
    pub script_path: &'a str,
}

/// Current code-freeze observations acquired by the process/filesystem adapter.
#[derive(Debug, Clone)]
pub struct HpcCodeFreezeObservation {
    pub slurm_script: String,
    pub git_head: Option<String>,
    pub git_branch: Option<String>,
    pub git_status_porcelain: Option<String>,
    pub dvc_available: bool,
}

/// Foundation references supplied by the campaign layout adapter.
#[derive(Debug, Clone)]
pub struct HpcBundleFoundationLocks {
    pub corpus_lock: String,
    pub database_lock: String,
    pub image_lock: String,
}
