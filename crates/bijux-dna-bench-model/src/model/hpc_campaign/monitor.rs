//! Local campaign observations; transport marker flags do not attest confidentiality or integrity.
use serde::Serialize;
pub const CAMPAIGN_MONITOR_SCHEMA_VERSION: &str = "bijux.hpc.slurm.submission.v1";
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SlurmMonitorReport {
    #[schemars(schema_with = "monitor_version_schema")]
    pub schema_version: &'static str,
    pub campaign_id: String,
    pub domain: String,
    pub snapshot: SlurmMonitorSnapshot,
    pub jobs: Vec<SlurmMonitorEntry>,
}

#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SlurmMonitorSnapshot {
    pub total_jobs: usize,
    pub jobs_with_log: usize,
    pub jobs_with_out: usize,
    pub jobs_with_err: usize,
    pub jobs_with_results_bundle: usize,
    pub jobs_with_code_bundle: usize,
    pub jobs_with_appraiser_done: usize,
}

#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
#[allow(clippy::struct_excessive_bools)]
#[serde(deny_unknown_fields)]
pub struct SlurmMonitorEntry {
    pub planned_job_id: String,
    pub scheduler_job_id: String,
    pub stage: String,
    pub tool: String,
    pub sample: String,
    pub log_exists: bool,
    pub out_exists: bool,
    pub err_exists: bool,
    pub results_exists: bool,
    pub results_sidecar_exists: bool,
    pub results_bundle_encrypted: bool,
    pub code_exists: bool,
    pub code_sidecar_exists: bool,
    pub code_bundle_encrypted: bool,
    pub appraiser_done: bool,
}

fn monitor_version_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
    schemars::json_schema!({"type":"string", "const": CAMPAIGN_MONITOR_SCHEMA_VERSION})
}
