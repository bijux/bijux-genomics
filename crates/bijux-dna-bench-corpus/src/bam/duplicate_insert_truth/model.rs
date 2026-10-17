//! Duplicate family and insert-size fixture declaration and report wire.
use bijux_dna_domain_bam::{BamDuplicateFamilyHistogramBinV1, BamInsertSizeHistogramBinV1};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct BamDuplicateInsertTruthManifest {
    pub(super) schema_version: String,
    pub(super) fixture_id: String,
    pub(super) description: String,
    pub(super) corpus_manifest_path: PathBuf,
    pub(super) expected_path: PathBuf,
    pub(super) source_paths: Vec<PathBuf>,
    pub(super) sample_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BamDuplicateInsertTruthValidationReport {
    pub schema_version: &'static str,
    pub fixture_id: String,
    pub manifest_path: String,
    pub expected_path: String,
    pub validated_sample_count: usize,
    pub validated_duplicate_family_bin_count: usize,
    pub validated_insert_size_bin_count: usize,
    pub valid: bool,
    pub checked_samples: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(super) struct BamDuplicateInsertTruthBundle {
    pub(super) schema_version: String,
    pub(super) fixture_id: String,
    pub(super) sample_truths: Vec<BamDuplicateInsertSampleTruth>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(super) struct BamDuplicateInsertSampleTruth {
    pub(super) sample_id: String,
    pub(super) cohort: String,
    pub(super) alignment_path: String,
    pub(super) total_reads: u64,
    pub(super) mapped_reads: u64,
    pub(super) unmapped_reads: u64,
    pub(super) examined_reads: u64,
    pub(super) duplicate_reads: u64,
    pub(super) duplicate_pairs: u64,
    pub(super) duplicate_family_histogram: Vec<BamDuplicateFamilyHistogramBinV1>,
    pub(super) pair_count: u64,
    pub(super) insert_size_histogram: Vec<BamInsertSizeHistogramBinV1>,
    #[serde(default)]
    pub(super) median_insert_size: Option<f64>,
    #[serde(default)]
    pub(super) mean_insert_size: Option<f64>,
    #[serde(default)]
    pub(super) min_insert_size: Option<u64>,
    #[serde(default)]
    pub(super) max_insert_size: Option<u64>,
}
