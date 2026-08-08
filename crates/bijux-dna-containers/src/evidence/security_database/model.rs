use serde::{Deserialize, Serialize};

pub const GRYPE_DATABASE_CONTRACT_SCHEMA_V1: &str = "bijux.container.grype_database_contract.v1";
pub const GRYPE_DATABASE_COMPLETION_SCHEMA_V1: &str = "bijux.grype_database_acquisition.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GrypeDatabaseContract {
    pub schema_version: String,
    pub acquisition_script_sha256: String,
    pub grype_version: String,
    pub grype_sha256: String,
    pub update_url: String,
    pub schema_major: u16,
    pub maximum_age_days: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GrypeDatabaseCompletion {
    pub schema_version: String,
    pub attempt_id: String,
    pub slurm_job_id: String,
    pub completed_at: String,
    pub grype_version: String,
    pub grype_sha256: String,
    pub status_sha256: String,
    pub database_manifest_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GrypeDatabaseStatus {
    pub schema_version: String,
    pub from: String,
    pub built: String,
    pub path: String,
    pub valid: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AdmittedGrypeDatabase {
    pub completion: GrypeDatabaseCompletion,
    pub status: GrypeDatabaseStatus,
    pub database_sha256: String,
}
