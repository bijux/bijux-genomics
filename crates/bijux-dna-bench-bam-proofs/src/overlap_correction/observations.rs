// Native records retain their full persisted input shape, including unconsumed diagnostics.
use serde::Deserialize;
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code, reason = "retain native deserialization contract fields")]
pub(super) struct LocalOverlapCorrectionSmokeReport {
    pub(super) schema_version: String,
    pub(super) stage_id: String,
    pub(super) sample_id: String,
    pub(super) expectation_matched: bool,
    pub(super) input_bam: String,
    pub(super) overlap_corrected_bam: String,
    pub(super) method: String,
    pub(super) pair_count: u64,
    pub(super) corrected_pairs: u64,
    pub(super) corrected_overlap_bases: u64,
    pub(super) insufficiency_reason: Option<String>,
    pub(super) overlap_correction_summary: String,
    pub(super) flagstat_before: String,
    pub(super) flagstat_after: String,
    pub(super) idxstats_before: String,
    pub(super) idxstats_after: String,
    pub(super) stage_metrics: String,
}
