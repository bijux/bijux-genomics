//! Declared population fixture wire records and their typed validation report.
use bijux_dna_domain_vcf::{
    VcfAdmixtureOutputTruthSummaryV1, VcfPcaOutputTruthSummaryV1,
    VcfPopulationStructureOutputTruthSummaryV1,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PopulationStructureTruthManifest {
    pub(super) schema_version: String,
    pub(super) fixture_id: String,
    pub(super) description: String,
    pub(super) expected_path: PathBuf,
    pub(super) source_paths: Vec<PathBuf>,
    pub(super) cases: Vec<PopulationStructureTruthCase>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PopulationStructureTruthCase {
    pub(super) case_id: String,
    pub(super) stage_id: String,
    pub(super) tool_id: String,
    pub(super) artifact_root_path: PathBuf,
    pub(super) sample_metadata_path: PathBuf,
}

#[derive(Debug, Clone, Serialize)]
pub struct PopulationStructureTruthValidationReport {
    pub schema_version: &'static str,
    pub fixture_id: String,
    pub manifest_path: String,
    pub expected_path: String,
    pub validated_case_count: usize,
    pub validated_stage_ids: Vec<String>,
    pub validated_tool_ids: Vec<String>,
    pub valid: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(super) struct PopulationStructureTruthBundle {
    pub(super) schema_version: String,
    pub(super) fixture_id: String,
    pub(super) cases: Vec<PopulationStructureTruthCaseTruth>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(super) struct PopulationStructureTruthCaseTruth {
    pub(super) case_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) pca_metrics: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) admixture_metrics: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) population_structure_metrics: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) pca_output_truth: Option<VcfPcaOutputTruthSummaryV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) admixture_output_truth: Option<VcfAdmixtureOutputTruthSummaryV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) population_structure_output_truth:
        Option<VcfPopulationStructureOutputTruthSummaryV1>,
}
