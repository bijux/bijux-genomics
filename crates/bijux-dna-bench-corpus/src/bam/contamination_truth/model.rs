//! Contamination fixture declarations, expected records and validation report wire.
use std::path::PathBuf;

use bijux_dna_domain_bam::BamAdnaContaminationTruthSummaryV1;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AdnaContaminationTruthManifest {
    pub(super) schema_version: String,
    pub(super) fixture_id: String,
    pub(super) description: String,
    pub(super) expected_path: PathBuf,
    pub(super) source_paths: Vec<PathBuf>,
    pub(super) cases: Vec<AdnaContaminationTruthCase>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AdnaContaminationTruthCase {
    pub(super) case_id: String,
    pub(super) sample_id: String,
    pub(super) cohort: String,
    pub(super) tool: String,
    pub(super) alignment_path: PathBuf,
    pub(super) reference_path: PathBuf,
    pub(super) minimum_mean_coverage: f64,
    pub(super) observed_mean_coverage: f64,
    pub(super) raw_estimate: f64,
    pub(super) raw_ci_low: f64,
    pub(super) raw_ci_high: f64,
    #[serde(default, flatten)]
    pub(super) context: AdnaContaminationTruthCaseContextSignals,
    #[serde(default, flatten)]
    pub(super) compatibility: AdnaContaminationTruthCaseCompatibilitySignals,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AdnaContaminationTruthCaseContextSignals {
    #[serde(default)]
    pub(super) has_mito_reference: bool,
    #[serde(default)]
    pub(super) has_damage_context: bool,
    #[serde(default)]
    pub(super) has_reference_panel: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AdnaContaminationTruthCaseCompatibilitySignals {
    #[serde(default)]
    pub(super) panel_build_compatible: bool,
    #[serde(default)]
    pub(super) sex_context_available: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct AdnaContaminationTruthValidationReport {
    pub schema_version: &'static str,
    pub fixture_id: String,
    pub manifest_path: String,
    pub expected_path: String,
    pub validated_case_count: usize,
    pub validated_insufficient_case_count: usize,
    pub tool_ids: Vec<String>,
    pub checked_cases: Vec<String>,
    pub valid: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(super) struct AdnaContaminationTruthBundle {
    pub(super) schema_version: String,
    pub(super) fixture_id: String,
    pub(super) truths: Vec<AdnaContaminationSampleTruth>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(super) struct AdnaContaminationSampleTruth {
    pub(super) case_id: String,
    pub(super) sample_id: String,
    pub(super) cohort: String,
    pub(super) tool: String,
    pub(super) alignment_path: String,
    pub(super) reference_path: String,
    pub(super) summary: BamAdnaContaminationTruthSummaryV1,
}
