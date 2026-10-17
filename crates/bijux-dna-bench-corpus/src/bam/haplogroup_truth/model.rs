//! Typed declarations, expected records and stable validation report.
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct BamHaplogroupTruthManifest {
    pub(super) schema_version: String,
    pub(super) fixture_id: String,
    pub(super) description: String,
    pub(super) corpus_manifest_path: PathBuf,
    pub(super) expected_path: PathBuf,
    pub(super) source_paths: Vec<PathBuf>,
    pub(super) cases: Vec<BamHaplogroupTruthCase>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct BamHaplogroupTruthCase {
    pub(super) case_id: String,
    pub(super) sample_id: String,
    pub(super) method: String,
    pub(super) reference_panel_id: String,
    pub(super) reference_build: String,
    pub(super) population_scope: String,
    pub(super) minimum_coverage: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct BamHaplogroupTruthValidationReport {
    pub schema_version: &'static str,
    pub fixture_id: String,
    pub manifest_path: String,
    pub expected_path: String,
    pub validated_case_count: usize,
    pub validated_ready_case_count: usize,
    pub validated_uncertain_case_count: usize,
    pub validated_coverage_gate_case_count: usize,
    pub validated_statuses: Vec<String>,
    pub checked_cases: Vec<String>,
    pub valid: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(super) struct BamHaplogroupTruthBundle {
    pub(super) schema_version: String,
    pub(super) fixture_id: String,
    pub(super) case_truths: Vec<BamHaplogroupCaseTruth>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(super) struct BamHaplogroupCaseTruth {
    pub(super) case_id: String,
    pub(super) sample_id: String,
    pub(super) cohort: String,
    pub(super) alignment_path: String,
    pub(super) reference_panel_path: String,
    pub(super) summary: BamHaplogroupTruthSummary,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(super) struct BamHaplogroupTruthSummary {
    pub(super) summary_schema_version: String,
    pub(super) stage_id: String,
    pub(super) method: String,
    pub(super) reference_panel_id: String,
    pub(super) reference_build: String,
    pub(super) population_scope: String,
    pub(super) minimum_coverage: f64,
    pub(super) observed_mean_coverage: f64,
    pub(super) ready: bool,
    #[serde(default)]
    pub(super) haplogroup_call: Option<String>,
    pub(super) confidence: f64,
    pub(super) status: String,
    pub(super) markers_total: u64,
    pub(super) markers_supported: u64,
    #[serde(default)]
    pub(super) supported_marker_ids: Vec<String>,
    #[serde(default)]
    pub(super) lineage_scope: Option<String>,
    #[serde(default)]
    pub(super) refusal_codes: Vec<String>,
}
