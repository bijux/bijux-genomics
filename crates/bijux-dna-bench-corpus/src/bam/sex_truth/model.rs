//! Typed declarations, expected records and stable validation report.
use bijux_dna_domain_bam::metrics::SexConfidenceClass;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct BamSexTruthManifest {
    pub(super) schema_version: String,
    pub(super) fixture_id: String,
    pub(super) description: String,
    pub(super) corpus_manifest_path: PathBuf,
    pub(super) expected_path: PathBuf,
    pub(super) source_paths: Vec<PathBuf>,
    pub(super) cases: Vec<BamSexTruthCase>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct BamSexTruthCase {
    pub(super) sample_id: String,
    pub(super) method: String,
    pub(super) chromosome_system: String,
    pub(super) minimum_y_sites: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct BamSexTruthValidationReport {
    pub schema_version: &'static str,
    pub fixture_id: String,
    pub manifest_path: String,
    pub expected_path: String,
    pub validated_case_count: usize,
    pub validated_ok_case_count: usize,
    pub validated_insufficient_case_count: usize,
    pub validated_call_classes: Vec<String>,
    pub checked_samples: Vec<String>,
    pub valid: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(super) struct BamSexTruthBundle {
    pub(super) schema_version: String,
    pub(super) fixture_id: String,
    pub(super) sample_truths: Vec<BamSexSampleTruth>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(super) struct BamSexSampleTruth {
    pub(super) sample_id: String,
    pub(super) cohort: String,
    pub(super) alignment_path: String,
    pub(super) reference_path: String,
    pub(super) summary: BamSexTruthSummary,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(super) struct BamSexTruthSummary {
    pub(super) summary_schema_version: String,
    pub(super) stage_id: String,
    pub(super) method: String,
    #[serde(default)]
    pub(super) chromosome_system: Option<String>,
    #[serde(default)]
    pub(super) minimum_y_sites: Option<u32>,
    pub(super) x_contig: String,
    pub(super) y_contig: String,
    pub(super) autosomal_contigs: Vec<String>,
    pub(super) x_coverage: f64,
    pub(super) y_coverage: f64,
    pub(super) autosomal_coverage: f64,
    pub(super) x_covered_sites: u64,
    pub(super) y_covered_sites: u64,
    #[serde(default)]
    pub(super) x_to_y_ratio: Option<f64>,
    pub(super) call: SexConfidenceClass,
    pub(super) confidence: f64,
    pub(super) status: String,
    #[serde(default)]
    pub(super) insufficiency_reason: Option<String>,
}
