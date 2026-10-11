//! Persisted BAM alignment reports and schema identities.

use super::identity::{BamArtifactInventoryV1, BamSampleIdentityV1};
use crate::params::ReadGroupSpec;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const BAM_ALIGNMENT_PROVENANCE_SCHEMA_VERSION: &str = "bijux.bam.alignment_provenance.v1";

pub const BAM_VALIDATION_SUMMARY_SCHEMA_VERSION: &str = "bijux.bam.validate.v1";

pub const BAM_QC_PRE_SUMMARY_SCHEMA_VERSION: &str = "bijux.bam.qc_pre.v1";

pub const BAM_MAPPING_SUMMARY_SCHEMA_VERSION: &str = "bijux.bam.mapping_summary.v1";

pub const BAM_ALIGNMENT_TRUTH_SUMMARY_SCHEMA_VERSION: &str = "bijux.bam.alignment_truth.v1";

pub const BAM_ALIGNMENT_STRATEGY_SCHEMA_VERSION: &str = "bijux.bam.alignment_strategy.v1";

pub const BAM_MERGE_COMPATIBILITY_SCHEMA_VERSION: &str = "bijux.bam.merge_compatibility.v1";

pub const BAM_POST_ALIGNMENT_CHAIN_SCHEMA_VERSION: &str = "bijux.bam.post_alignment_chain.v1";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BamFlagstatCountsV1 {
    #[serde(default)]
    pub total_reads: Option<u64>,
    #[serde(default)]
    pub mapped_reads: Option<u64>,
    #[serde(default)]
    pub duplicate_reads: Option<u64>,
    #[serde(default)]
    pub mapped_fraction: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BamValidationSummaryV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub input_bam: PathBuf,
    #[serde(default)]
    pub bam_index: Option<PathBuf>,
    #[serde(default)]
    pub reference_fasta: Option<PathBuf>,
    pub flagstat: BamFlagstatCountsV1,
    pub validation_report_present: bool,
    #[serde(default)]
    pub refusal_codes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BamTinyAlignmentInspectionV1 {
    pub sort_order: Option<String>,
    pub header_contigs: Vec<String>,
    pub header_sample_ids: Vec<String>,
    pub read_group_ids: Vec<String>,
    pub mapped_record_contigs: Vec<String>,
    pub record_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BamAlignmentTruthRecordV1 {
    pub qname: String,
    pub mapped: bool,
    pub reference_name: Option<String>,
    pub position: Option<u64>,
    pub mapq: u8,
    pub cigar: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BamAlignmentTruthPositionV1 {
    pub reference_name: String,
    pub position: u64,
    pub read_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BamAlignmentTruthMapqClassV1 {
    pub mapq: u8,
    pub read_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BamAlignmentTruthCigarClassV1 {
    pub cigar: String,
    pub read_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BamAlignmentTruthSummaryV1 {
    pub schema_version: String,
    pub total_reads: u64,
    pub mapped_reads: u64,
    pub unmapped_reads: u64,
    pub mapped_contigs: Vec<String>,
    pub mapped_positions: Vec<BamAlignmentTruthPositionV1>,
    pub mapped_mapq_classes: Vec<BamAlignmentTruthMapqClassV1>,
    pub cigar_classes: Vec<BamAlignmentTruthCigarClassV1>,
    pub records: Vec<BamAlignmentTruthRecordV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BamQcPreSummaryV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub input_bam: PathBuf,
    pub total_reads: u64,
    pub mapped_reads: u64,
    pub unmapped_reads: u64,
    pub duplicate_flagged_reads: u64,
    pub contig_summary: Vec<crate::metrics::IdxstatsContigV1>,
    pub reference_mismatch: bool,
    pub fragment_length: crate::metrics::FragmentLengthSummaryV1,
    pub mapq: crate::metrics::MapqSummaryV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BamMapqRegimeV1 {
    pub mean: f64,
    pub warn_below: f64,
    pub fail_below: f64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BamMappingSummaryV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub flagstat: BamFlagstatCountsV1,
    pub stats_present: bool,
    pub idxstats_present: bool,
    #[serde(default)]
    pub mapq_regime: Option<BamMapqRegimeV1>,
    #[serde(default)]
    pub proper_pair_reads: Option<u64>,
    #[serde(default)]
    pub secondary_reads: Option<u64>,
    #[serde(default)]
    pub supplementary_reads: Option<u64>,
    #[serde(default)]
    pub mapq_histogram: Vec<(u8, u64)>,
    #[serde(default)]
    pub read_group_breakdown: Vec<BamReadGroupMappingCountV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BamReadGroupMappingCountV1 {
    pub read_group_id: String,
    pub total_reads: u64,
    pub mapped_reads: u64,
    pub proper_pair_reads: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BamAlignmentSuitabilityV1 {
    GeneralShortRead,
    WholeGenomeLike,
    TargetedCapture,
    AncientShortFragments,
    SensitiveLocalRescue,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BamPostAlignmentChainV1 {
    pub schema_version: String,
    pub chain_id: String,
    pub sorting_mode: String,
    pub index_type: String,
    pub validate_before_downstream: bool,
    pub coordinate_sorted_required_for: Vec<String>,
    pub downstream_refusal_stages: Vec<String>,
    pub required_tools: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BamAlignmentStrategyV1 {
    pub schema_version: String,
    pub strategy_id: String,
    pub tool_id: String,
    pub default_preset: String,
    pub mode: String,
    pub supported_inputs: Vec<String>,
    pub required_reference_assets: Vec<String>,
    pub suitability: Vec<BamAlignmentSuitabilityV1>,
    pub hidden_default_allowed: bool,
    pub post_alignment_chain: BamPostAlignmentChainV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BamMergeInputIdentityV1 {
    pub sample_id: String,
    #[serde(default)]
    pub read_group_ids: Vec<String>,
    #[serde(default)]
    pub reference_digest: Option<String>,
    #[serde(default)]
    pub sequencing_platform: Option<String>,
    #[serde(default)]
    pub library_id: Option<String>,
    #[serde(default)]
    pub lane_id: Option<String>,
    #[serde(default)]
    pub platform_unit: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BamMergeCompatibilityV1 {
    pub schema_version: String,
    pub compatible: bool,
    #[serde(default)]
    pub merged_sample_id: Option<String>,
    #[serde(default)]
    pub refusal_codes: Vec<String>,
    #[serde(default)]
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BamAlignmentProvenanceV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub backend_tool_id: String,
    #[serde(default)]
    pub strategy_id: Option<String>,
    #[serde(default)]
    pub preset: Option<String>,
    #[serde(default)]
    pub mode: Option<String>,
    #[serde(default)]
    pub sensitivity_profile: Option<String>,
    #[serde(default)]
    pub seed_length: Option<u32>,
    pub reference_fasta: PathBuf,
    #[serde(default)]
    pub reference_digest: Option<String>,
    #[serde(default)]
    pub post_alignment_chain: Option<BamPostAlignmentChainV1>,
    pub sample_identity: BamSampleIdentityV1,
    pub read_group: ReadGroupSpec,
    pub outputs: BamArtifactInventoryV1,
}
