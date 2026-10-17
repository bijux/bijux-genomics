//! Wire records for declared filter fixture expectations and validation reports.
use bijux_dna_domain_vcf::VcfFilterOutputTruthSummaryV1;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct VcfFilterTruthManifest {
    pub(super) schema_version: String,
    pub(super) fixture_id: String,
    pub(super) description: String,
    pub(super) expected_path: PathBuf,
    pub(super) source_paths: Vec<PathBuf>,
    pub(super) cases: Vec<VcfFilterTruthCase>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct VcfFilterTruthCase {
    pub(super) case_id: String,
    pub(super) stage_id: String,
    pub(super) tool_id: String,
    pub(super) artifact_root_path: PathBuf,
    #[serde(default)]
    pub(super) output_vcf_path: Option<PathBuf>,
    pub(super) truth_kind: VcfFilterTruthKind,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(super) enum VcfFilterTruthKind {
    FilterLabels,
    DamageRemoval,
}

#[derive(Debug, Clone, Serialize)]
pub struct VcfFilterTruthValidationReport {
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
pub(super) struct VcfFilterTruthBundle {
    pub(super) schema_version: String,
    pub(super) fixture_id: String,
    pub(super) cases: Vec<VcfFilterTruthCaseTruth>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(super) struct VcfFilterTruthCaseTruth {
    pub(super) case_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) filter_metrics: Option<VcfFilterMetricsTruth>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) damage_filter_metrics: Option<VcfDamageFilterMetricsTruth>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) output_truth: Option<VcfFilterOutputTruthSummaryV1>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(super) struct VcfFilterMetricsTruth {
    pub(super) schema_version: String,
    pub(super) stage_id: String,
    pub(super) tool_id: String,
    pub(super) input_variants: u64,
    pub(super) pass_variants: u64,
    pub(super) failed_variants: u64,
    pub(super) filter_ids: Vec<String>,
    pub(super) depth_threshold: f64,
    pub(super) quality_threshold: f64,
    pub(super) missingness_threshold: f64,
    pub(super) sample_count: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(super) struct VcfDamageFilterMetricsTruth {
    pub(super) schema_version: String,
    pub(super) stage_id: String,
    pub(super) tool_id: String,
    pub(super) input_variants: u64,
    pub(super) removed_variants: u64,
    pub(super) retained_variants: u64,
    pub(super) low_quality_filtered_variants: u64,
    pub(super) damage_ratio_filtered_variants: u64,
    pub(super) terminal_damage_filtered_variants: u64,
    pub(super) damage_context_rule: String,
    pub(super) terminal_context_count: u64,
    pub(super) sample_count: u64,
}
