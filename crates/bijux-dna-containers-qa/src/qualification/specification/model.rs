//! Qualification specification input, selection and coverage wire authorities.
use schemars::{schema_for, JsonSchema, Schema};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::PathBuf};

pub const QUALIFICATION_SPECIFICATION_SOURCE_SCHEMA_V1: &str =
    "bijux.container.qualification_specification_source.v1";
pub const QUALIFICATION_SPECIFICATION_COVERAGE_SCHEMA_V2: &str =
    "bijux.container.qualification_specification_coverage.v2";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QualificationSpecificationCompilationRequest {
    pub repository_root: PathBuf,
    pub registry_path: PathBuf,
    pub inventory_path: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QualificationSpecificationInputKindV1 {
    Data,
    Reference,
    Database,
    UpstreamResult,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct QualificationSpecificationInputV1 {
    pub binding_input: String,
    pub artifact_contract_id: String,
    pub id: String,
    pub kind: QualificationSpecificationInputKindV1,
    pub path: PathBuf,
    pub mount_path: PathBuf,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct QualificationSpecificationOutputV1 {
    pub binding_output: String,
    pub artifact_contract_id: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct QualificationSpecificationSiteResourcesV1 {
    pub account: String,
    pub partition: String,
    pub walltime: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct QualificationSpecificationSourceV1 {
    pub schema_version: String,
    pub catalog_sha256: String,
    pub registry_sha256: String,
    pub inventory_sha256: String,
    pub governed_study_root: PathBuf,
    pub repository_path: PathBuf,
    pub repository_commit: String,
    pub run_id: String,
    pub tool_id: String,
    pub campaign: String,
    pub corpus: String,
    pub corpus_role: String,
    pub sample: String,
    pub operator: String,
    pub submitted_at_utc: String,
    pub result_root: PathBuf,
    pub inputs: Vec<QualificationSpecificationInputV1>,
    #[serde(default)]
    pub parameters: BTreeMap<String, String>,
    pub outputs: Vec<QualificationSpecificationOutputV1>,
    pub negative_command: Vec<String>,
    pub site: QualificationSpecificationSiteResourcesV1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QualificationSpecificationCoverageState {
    MissingBinding,
    ImageUnavailable,
    SourceMissing,
    InvalidSource,
    Ready,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QualificationSpecificationCoverageUnit {
    pub run_id: String,
    pub tool_id: String,
    pub stage_id: String,
    pub state: QualificationSpecificationCoverageState,
    pub binding_ready: bool,
    pub image_ready: bool,
    pub source_present: bool,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QualificationSpecificationCoverageSummary {
    pub supported_tool_count: usize,
    pub binding_ready_count: usize,
    pub image_ready_count: usize,
    pub source_present_count: usize,
    pub compile_ready_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QualificationSpecificationCoverage {
    pub schema_version: String,
    pub catalog_sha256: String,
    pub registry_sha256: String,
    pub inventory_sha256: String,
    pub summary: QualificationSpecificationCoverageSummary,
    pub units: Vec<QualificationSpecificationCoverageUnit>,
}

/// Return the generated schema for declarative qualification compiler inputs.
#[must_use]
pub fn qualification_specification_source_schema() -> Schema {
    schema_for!(QualificationSpecificationSourceV1)
}

pub const QUALIFICATION_SOURCE_SELECTION_SCHEMA_V1: &str =
    "bijux.container.qualification_source_selection.v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QualificationSourceMaterializationRequest {
    pub repository_root: PathBuf,
    pub registry_path: PathBuf,
    pub inventory_path: PathBuf,
    pub data_index_path: PathBuf,
    pub governed_study_root: PathBuf,
    pub repository_commit: String,
    pub operator: String,
    pub submitted_at_utc: String,
    pub account: String,
    pub partition: String,
    pub walltime: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct QualificationSourceInputSelectionV1 {
    pub binding_input: String,
    pub id: String,
    pub kind: QualificationSpecificationInputKindV1,
    pub catalog_id: String,
    pub study_uri: String,
    pub mount_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct QualificationSourceSelectionV1 {
    pub schema_version: String,
    pub tool_id: String,
    pub campaign: String,
    pub corpus: String,
    pub corpus_role: String,
    pub sample: String,
    pub inputs: Vec<QualificationSourceInputSelectionV1>,
    #[serde(default)]
    pub parameters: BTreeMap<String, String>,
    pub negative_arguments: Vec<String>,
}

/// Return the generated schema for declarative qualification data selections.
#[must_use]
pub fn qualification_source_selection_schema() -> Schema {
    schema_for!(QualificationSourceSelectionV1)
}

#[cfg(test)]
#[path = "../../../tests/contracts/qualification_specification.rs"]
mod tests;
