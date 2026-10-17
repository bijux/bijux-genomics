//! Typed declarations, expected records and stable validation report.
use bijux_dna_domain_bam::{BamCoverageTruthSummaryV1, BamGcBiasTruthSummaryV1};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct BamGcCoverageTruthManifest {
    pub(super) schema_version: String,
    pub(super) fixture_id: String,
    pub(super) description: String,
    pub(super) corpus_manifest_path: PathBuf,
    pub(super) expected_path: PathBuf,
    pub(super) source_paths: Vec<PathBuf>,
    pub(super) coverage_cases: Vec<BamCoverageTruthCase>,
    pub(super) gc_bias_cases: Vec<BamGcBiasTruthCase>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct BamCoverageTruthCase {
    pub(super) sample_id: String,
    pub(super) regions_path: PathBuf,
    pub(super) depth_thresholds: Vec<u32>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct BamGcBiasTruthCase {
    pub(super) sample_id: String,
    pub(super) reference_path: PathBuf,
    pub(super) window_size: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct BamGcCoverageTruthValidationReport {
    pub schema_version: &'static str,
    pub fixture_id: String,
    pub manifest_path: String,
    pub expected_path: String,
    pub validated_coverage_sample_count: usize,
    pub validated_gc_bias_sample_count: usize,
    pub validated_coverage_region_count: usize,
    pub validated_gc_bias_bin_count: usize,
    pub valid: bool,
    pub checked_samples: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(super) struct BamGcCoverageTruthBundle {
    pub(super) schema_version: String,
    pub(super) fixture_id: String,
    pub(super) coverage_truths: Vec<BamCoverageSampleTruth>,
    pub(super) gc_bias_truths: Vec<BamGcBiasSampleTruth>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(super) struct BamCoverageSampleTruth {
    pub(super) sample_id: String,
    pub(super) cohort: String,
    pub(super) alignment_path: String,
    pub(super) regions_path: String,
    pub(super) summary: BamCoverageTruthSummaryV1,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(super) struct BamGcBiasSampleTruth {
    pub(super) sample_id: String,
    pub(super) cohort: String,
    pub(super) alignment_path: String,
    pub(super) reference_path: String,
    pub(super) summary: BamGcBiasTruthSummaryV1,
}
