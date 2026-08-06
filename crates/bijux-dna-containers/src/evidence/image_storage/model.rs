use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::super::{ContainerImageInventorySummary, EvidenceSourceIdentity};

pub const IMAGE_STORAGE_PLAN_SCHEMA_V1: &str = "bijux.container.image_storage_plan.v1";
pub const IMAGE_STORAGE_REPORT_SCHEMA_V1: &str = "bijux.container.image_storage_report.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ImageStoragePlan {
    pub schema_version: String,
    pub state: ImageStoragePlanState,
    pub created_at_utc: String,
    pub inventory: EvidenceSourceIdentity,
    pub image_root: String,
    pub quarantine_root: String,
    pub file_count: u64,
    pub size_bytes: u64,
    pub moves: Vec<ImageStorageMove>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at_utc: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rolled_back_at_utc: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ImageStoragePlanState {
    Previewed,
    Applying,
    Applied,
    RollingBack,
    RolledBack,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ImageStorageMove {
    pub category: ImageStorageMoveCategory,
    pub source: String,
    pub target: String,
    pub size_bytes: u64,
    pub sha256: String,
    pub applied: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub applied_at_utc: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ImageStorageMoveCategory {
    Historical,
    Obsolete,
    Interrupted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ImageStorageReport {
    pub schema_version: String,
    pub repository_commit: String,
    pub verdict: ImageStorageVerdict,
    pub moved_file_count: u64,
    pub moved_size_bytes: u64,
    pub post_storage_summary: ContainerImageInventorySummary,
    pub checks: BTreeMap<String, bool>,
    pub historical_dependencies_preserved: u64,
    pub recovery: ImageStorageRecovery,
    pub source_artifacts: ImageStorageReportSources,
    pub moved_images: Vec<QuarantinedImageIdentity>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ImageStorageVerdict {
    Passed,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ImageStorageRecovery {
    pub mode: String,
    pub command: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ImageStorageReportSources {
    pub storage_plan: ImageStorageReportSource,
    pub post_storage_inventory: ImageStorageReportSource,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ImageStorageReportSource {
    pub study_relative_path: String,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct QuarantinedImageIdentity {
    pub category: ImageStorageMoveCategory,
    pub name: String,
    pub sha256: String,
    pub size_bytes: u64,
    pub quarantine_path: String,
}
