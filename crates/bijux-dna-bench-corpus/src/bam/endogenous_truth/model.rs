//! Typed declarations, expected records and stable validation report.
use bijux_dna_domain_bam::BamEndogenousTruthSummaryV1;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct BamEndogenousTruthManifest {
    pub(super) schema_version: String,
    pub(super) fixture_id: String,
    pub(super) description: String,
    pub(super) corpus_manifest_path: PathBuf,
    pub(super) expected_path: PathBuf,
    pub(super) source_paths: Vec<PathBuf>,
    pub(super) cases: Vec<BamEndogenousTruthCase>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct BamEndogenousTruthCase {
    pub(super) sample_id: String,
    pub(super) method: String,
    pub(super) host_reference_scope: String,
    pub(super) prealignment_fraction: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct BamEndogenousTruthValidationReport {
    pub schema_version: &'static str,
    pub fixture_id: String,
    pub manifest_path: String,
    pub expected_path: String,
    pub validated_sample_count: usize,
    pub validated_total_reads: u64,
    pub validated_contaminant_reads: u64,
    pub validated_retained_reads: u64,
    pub checked_samples: Vec<String>,
    pub valid: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(super) struct BamEndogenousTruthBundle {
    pub(super) schema_version: String,
    pub(super) fixture_id: String,
    pub(super) sample_truths: Vec<BamEndogenousTruthSample>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(super) struct BamEndogenousTruthSample {
    pub(super) sample_id: String,
    pub(super) cohort: String,
    pub(super) alignment_path: String,
    pub(super) summary: BamEndogenousTruthSummaryV1,
}
