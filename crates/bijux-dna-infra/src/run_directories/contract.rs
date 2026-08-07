use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct RunLayoutPaths {
    pub run_dir: PathBuf,
    pub artifacts_dir: PathBuf,
    pub logs_dir: PathBuf,
    pub tmp_dir: PathBuf,
}

#[derive(Debug, Clone, Copy)]
pub struct RunLayoutContract {
    pub runs_dir: &'static str,
    pub artifacts_dir: &'static str,
    pub logs_dir: &'static str,
    pub tmp_dir: &'static str,
    pub lock_file: &'static str,
    pub publish_marker: &'static str,
}

pub const RUN_LAYOUT_CONTRACT: RunLayoutContract = RunLayoutContract {
    runs_dir: "runs",
    artifacts_dir: "artifacts",
    logs_dir: "logs",
    tmp_dir: "tmp",
    lock_file: ".run.lock",
    publish_marker: "published.json",
};

pub const PIPELINE_RUN_DIR_TEMPLATE: &str = "{pipeline_id}/{sample_id}/{run_id}";

pub const ATTEMPT_LAYOUT_SCHEMA_V1: &str = "bijux.attempt_layout.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttemptEvidenceOwner {
    Benchmarks,
    Containers,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttemptLayoutIdentity {
    pub run_directory: String,
    pub tool_id: String,
    pub campaign_id: String,
    pub attempt_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttemptLayoutPaths {
    pub attempt_dir: PathBuf,
    pub manifests_dir: PathBuf,
    pub logs_dir: PathBuf,
    pub outputs_dir: PathBuf,
    pub accounting_dir: PathBuf,
    pub validation_dir: PathBuf,
    pub science_dir: PathBuf,
    pub receipts_dir: PathBuf,
    pub layout_manifest_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttemptLayoutDirectoriesV1 {
    pub manifests: String,
    pub logs: String,
    pub outputs: String,
    pub accounting: String,
    pub validation: String,
    pub science: String,
    pub receipts: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttemptLayoutManifestV1 {
    pub schema_version: String,
    pub owner: AttemptEvidenceOwner,
    pub run_directory: String,
    pub tool_id: String,
    pub campaign_id: String,
    pub attempt_id: String,
    pub directories: AttemptLayoutDirectoriesV1,
}

#[derive(Debug, Clone, Copy)]
pub struct AttemptLayoutContract {
    pub attempts_dir: &'static str,
    pub manifests_dir: &'static str,
    pub logs_dir: &'static str,
    pub outputs_dir: &'static str,
    pub accounting_dir: &'static str,
    pub validation_dir: &'static str,
    pub science_dir: &'static str,
    pub receipts_dir: &'static str,
    pub layout_manifest: &'static str,
}

pub const ATTEMPT_LAYOUT_CONTRACT: AttemptLayoutContract = AttemptLayoutContract {
    attempts_dir: "attempts",
    manifests_dir: "manifests",
    logs_dir: "logs",
    outputs_dir: "outputs",
    accounting_dir: "accounting",
    validation_dir: "validation",
    science_dir: "science",
    receipts_dir: "receipts",
    layout_manifest: "attempt-layout.json",
};
