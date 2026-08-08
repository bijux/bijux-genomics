use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

pub const SECURITY_SCANNER_TOOLCHAIN_CONTRACT_SCHEMA_V1: &str =
    "bijux.container.security_scanner_toolchain.v1";
pub const SECURITY_SCANNER_TOOLCHAIN_COMPLETION_SCHEMA_V1: &str =
    "bijux.security_scanner_toolchain_completion.v1";

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SecurityScannerToolchainContract {
    pub schema_version: String,
    pub acquisition_script_sha256: String,
    pub tools: Vec<SecurityScannerTool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SecurityScannerTool {
    pub id: String,
    pub version: String,
    pub archive: String,
    pub archive_sha256: String,
    pub checksums: String,
    pub checksums_sha256: String,
    pub release: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SecurityScannerToolchainCompletion {
    pub schema_version: String,
    pub attempt_id: String,
    pub slurm_job_id: String,
    pub completed_at: String,
    pub binaries: BTreeMap<String, SecurityScannerBinary>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SecurityScannerBinary {
    pub version: String,
    pub sha256: String,
}
