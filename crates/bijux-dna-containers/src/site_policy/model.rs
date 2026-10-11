use std::path::PathBuf;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const CONTAINER_GOVERNANCE_SCHEMA_V1: &str = "bijux.container.governance.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContainerGovernance {
    pub schema_version: String,
    pub site: ContainerSitePolicy,
    pub paths: ContainerPathPolicy,
    pub runtime: ContainerRuntimePolicy,
    pub slurm: ContainerSlurmPolicy,
    pub preflight: ContainerPreflightPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContainerSitePolicy {
    pub id: String,
    pub hostname_regex: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContainerPathPolicy {
    pub study_root: PathBuf,
    pub containers_root: PathBuf,
    pub data_root: PathBuf,
    pub results_root: PathBuf,
    pub finalization_root: PathBuf,
    pub active_repository: PathBuf,
    pub frozen_repositories: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContainerRuntimePolicy {
    pub apptainer_architecture: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContainerSlurmPolicy {
    pub account: String,
    pub scratch_environment: Option<String>,
    pub scratch_root: Option<PathBuf>,
    pub partitions: Vec<String>,
    pub quota_command: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContainerPreflightPolicy {
    pub minimum_available_bytes: u64,
    pub minimum_available_inodes: u64,
    pub maximum_clock_skew_seconds: u64,
    pub required_commands: Vec<String>,
    pub required_contracts: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RepositoryFreezeScope {
    Contracts,
    Sources,
    Complete,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepositoryFreezeRequest {
    pub governance: ContainerGovernance,
    pub hostname: String,
    pub commit: String,
    pub scope: RepositoryFreezeScope,
    pub checkout_paths: Vec<PathBuf>,
    pub source_namespace: String,
}

pub const REPOSITORY_FREEZE_ARCHIVE_RECEIPT_SCHEMA_V1: &str =
    "bijux.container.repository_freeze_archive_receipt.v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepositoryFreezeArchiveRequest {
    pub governance: ContainerGovernance,
    pub hostname: String,
    pub snapshot: PathBuf,
    pub commit: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RepositoryFreezeArchiveReceipt {
    pub schema_version: String,
    pub snapshot_relative_path: PathBuf,
    pub commit: String,
    pub bundle_path: PathBuf,
    pub bundle_sha256: String,
    pub tracked_index_sha256: String,
    pub sparse_checkout_paths: Vec<PathBuf>,
    pub restoration_verified: bool,
}
