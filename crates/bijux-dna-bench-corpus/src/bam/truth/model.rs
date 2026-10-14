//! Persisted BAM alignment truth wire records and validation observations.
use bijux_dna_domain_bam::{
    BamAlignmentTruthCigarClassV1, BamAlignmentTruthMapqClassV1, BamAlignmentTruthPositionV1,
    BamAlignmentTruthRecordV1,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct BamAlignmentTruthManifest {
    pub(super) schema_version: String,
    pub(super) fixture_id: String,
    pub(super) description: String,
    pub(super) corpus_manifest_path: PathBuf,
    pub(super) expected_path: PathBuf,
    pub(super) source_paths: Vec<PathBuf>,
    pub(super) sample_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BamAlignmentTruthValidationReport {
    pub schema_version: &'static str,
    pub fixture_id: String,
    pub manifest_path: String,
    pub expected_path: String,
    pub validated_sample_count: usize,
    pub validated_record_count: usize,
    pub valid: bool,
    pub checked_samples: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(super) struct BamAlignmentTruthBundle {
    pub(super) schema_version: String,
    pub(super) fixture_id: String,
    pub(super) sample_truths: Vec<BamAlignmentSampleTruth>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(super) struct BamAlignmentSampleTruth {
    pub(super) sample_id: String,
    pub(super) cohort: String,
    pub(super) alignment_path: String,
    pub(super) total_reads: u64,
    pub(super) mapped_reads: u64,
    pub(super) unmapped_reads: u64,
    pub(super) mapped_contigs: Vec<String>,
    pub(super) mapped_positions: Vec<BamAlignmentTruthPositionV1>,
    pub(super) mapped_mapq_classes: Vec<BamAlignmentTruthMapqClassV1>,
    pub(super) cigar_classes: Vec<BamAlignmentTruthCigarClassV1>,
    pub(super) records: Vec<BamAlignmentTruthRecordV1>,
}
