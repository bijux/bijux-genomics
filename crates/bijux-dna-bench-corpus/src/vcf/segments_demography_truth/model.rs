//! Declared segment fixture wire records and the typed admission report.
use bijux_dna_domain_vcf::{
    VcfDemographyOutputTruthSummaryV1, VcfIbdOutputTruthSummaryV1, VcfRohOutputTruthSummaryV1,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SegmentsDemographyTruthManifest {
    pub(super) schema_version: String,
    pub(super) fixture_id: String,
    pub(super) description: String,
    pub(super) expected_path: PathBuf,
    pub(super) source_paths: Vec<PathBuf>,
    pub(super) cases: Vec<SegmentsDemographyTruthCase>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SegmentsDemographyTruthCase {
    pub(super) case_id: String,
    pub(super) stage_id: String,
    pub(super) tool_id: String,
    pub(super) artifact_root_path: PathBuf,
}

#[derive(Debug, Clone, Serialize)]
pub struct SegmentsDemographyTruthValidationReport {
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
pub(super) struct SegmentsDemographyTruthBundle {
    pub(super) schema_version: String,
    pub(super) fixture_id: String,
    pub(super) cases: Vec<SegmentsDemographyTruthCaseTruth>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(super) struct SegmentsDemographyTruthCaseTruth {
    pub(super) case_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) roh_metrics: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) ibd_metrics: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) demography_metrics: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) roh_output_truth: Option<VcfRohOutputTruthSummaryV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) ibd_output_truth: Option<VcfIbdOutputTruthSummaryV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) demography_output_truth: Option<VcfDemographyOutputTruthSummaryV1>,
}
