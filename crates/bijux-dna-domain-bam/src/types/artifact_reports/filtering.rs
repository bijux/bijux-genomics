//! Persisted BAM filtering reports and schema identities.

use super::alignment::BamFlagstatCountsV1;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const BAM_FILTER_SUMMARY_SCHEMA_VERSION: &str = "bijux.bam.filter.v1";

pub const BAM_MAPQ_FILTER_SUMMARY_SCHEMA_VERSION: &str = "bijux.bam.mapq_filter.v1";

pub const BAM_LENGTH_FILTER_SUMMARY_SCHEMA_VERSION: &str = "bijux.bam.length_filter.v1";

pub const BAM_OVERLAP_CORRECTION_SUMMARY_SCHEMA_VERSION: &str = "bijux.bam.overlap_correction.v1";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BamFilterSummaryV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub input_bam: PathBuf,
    pub output_bam: PathBuf,
    pub flagstat_before: BamFlagstatCountsV1,
    pub flagstat_after: BamFlagstatCountsV1,
    pub input_reads: u64,
    pub kept_reads: u64,
    pub removed_reads: u64,
    #[serde(default)]
    pub active_filters: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BamMapqFilterSummaryV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub mapq_threshold: u8,
    pub input_bam: PathBuf,
    pub output_bam: PathBuf,
    pub flagstat_before: BamFlagstatCountsV1,
    pub flagstat_after: BamFlagstatCountsV1,
    pub input_reads: u64,
    pub kept_reads: u64,
    pub removed_reads: u64,
    #[serde(default)]
    pub mapped_reads_removed: Option<u64>,
    #[serde(default)]
    pub mapped_fraction_retained: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BamLengthFilterSummaryV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub min_length_threshold: u32,
    pub input_bam: PathBuf,
    pub output_bam: PathBuf,
    pub flagstat_before: BamFlagstatCountsV1,
    pub flagstat_after: BamFlagstatCountsV1,
    pub input_reads: u64,
    pub kept_reads: u64,
    pub removed_reads: u64,
    #[serde(default)]
    pub observed_min_length: Option<u32>,
    #[serde(default)]
    pub observed_max_length: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BamOverlapCorrectionSummaryV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub method: String,
    pub input_bam: PathBuf,
    pub output_bam: PathBuf,
    pub flagstat_before: BamFlagstatCountsV1,
    pub flagstat_after: BamFlagstatCountsV1,
    #[serde(default)]
    pub pair_count: Option<u64>,
    #[serde(default)]
    pub corrected_pairs: Option<u64>,
    #[serde(default)]
    pub corrected_overlap_bases: Option<u64>,
    #[serde(default)]
    pub insufficiency_reason: Option<String>,
}
