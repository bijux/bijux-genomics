//! Declared phasing/imputation wire records and their validation report.
use std::collections::BTreeMap;
use std::path::PathBuf;

use bijux_dna_domain_vcf::{VcfImputationOutputTruthSummaryV1, VcfPhasingOutputTruthSummaryV1};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PhasingImputationTruthManifest {
    pub(super) schema_version: String,
    pub(super) fixture_id: String,
    pub(super) description: String,
    pub(super) expected_path: PathBuf,
    pub(super) source_paths: Vec<PathBuf>,
    pub(super) cases: Vec<PhasingImputationTruthCase>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PhasingImputationTruthCase {
    pub(super) case_id: String,
    pub(super) stage_id: String,
    pub(super) tool_id: String,
    pub(super) artifact_root_path: PathBuf,
    pub(super) output_vcf_path: PathBuf,
    #[serde(default)]
    pub(super) truth_vcf_path: Option<PathBuf>,
    pub(super) truth_kind: PhasingImputationTruthKind,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(super) enum PhasingImputationTruthKind {
    PhasingOutput,
    ImputationOutput,
}

#[derive(Debug, Clone, Serialize)]
pub struct PhasingImputationTruthValidationReport {
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
pub(super) struct PhasingImputationTruthBundle {
    pub(super) schema_version: String,
    pub(super) fixture_id: String,
    pub(super) cases: Vec<PhasingImputationTruthCaseTruth>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(super) struct PhasingImputationTruthCaseTruth {
    pub(super) case_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) phasing_metrics: Option<VcfPhasingMetricsTruth>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) impute_metrics: Option<VcfImputeMetricsTruth>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) imputation_metrics: Option<VcfImputationMetricsTruth>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) phasing_output_truth: Option<VcfPhasingOutputTruthSummaryV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) imputation_output_truth: Option<VcfImputationOutputTruthSummaryV1>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(super) struct VcfPhasingMetricsTruth {
    pub(super) schema_version: String,
    pub(super) stage_id: String,
    pub(super) tool_id: String,
    pub(super) input_genotypes: u64,
    pub(super) phased_genotypes: u64,
    pub(super) unphased_genotypes: u64,
    pub(super) phase_set_count: u64,
    pub(super) sample_count: usize,
    pub(super) sample_ids: Vec<String>,
    pub(super) output_variant_count: u64,
    pub(super) status: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(super) struct VcfImputeMetricsTruth {
    pub(super) schema_version: String,
    pub(super) stage_id: String,
    pub(super) tool_id: String,
    pub(super) variant_count: u64,
    pub(super) missing_before: u64,
    pub(super) missing_after: u64,
    pub(super) imputed_genotypes: u64,
    pub(super) low_confidence_count: u64,
    pub(super) masked_truth_site_count: u64,
    pub(super) masked_truth_match_count: u64,
    pub(super) unresolved_count: u64,
    pub(super) not_imputable_reasons: BTreeMap<String, u64>,
    pub(super) sample_count: usize,
    pub(super) sample_ids: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(super) struct VcfImputationMetricsTruth {
    pub(super) schema_version: String,
    pub(super) stage_id: String,
    pub(super) tool_id: String,
    pub(super) status: String,
    pub(super) mean_info_score: f64,
    pub(super) r2_available: bool,
    pub(super) low_confidence_sites: u64,
    pub(super) masked_truth_sites: u64,
    pub(super) missing_quality_fields: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) concordance: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) dosage_r2: Option<f64>,
    pub(super) variant_count: u64,
    pub(super) sample_count: usize,
    pub(super) sample_ids: Vec<String>,
}
