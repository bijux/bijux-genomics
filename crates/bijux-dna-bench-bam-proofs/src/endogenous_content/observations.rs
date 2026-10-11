use serde::Deserialize;
#[derive(Debug, Clone, Deserialize)]
pub(super) struct LocalEndogenousContentSmokeReport {
    pub(super) schema_version: String,
    pub(super) stage_id: String,
    pub(super) sample_id: String,
    pub(super) expectation_matched: bool,
    pub(super) input_bam: String,
    pub(super) method: String,
    pub(super) host_reference_scope: Option<String>,
    pub(super) mapped_reads: u64,
    pub(super) endogenous_reads: u64,
    pub(super) total_reads: u64,
    pub(super) endogenous_fraction: f64,
    pub(super) prealignment_fraction: Option<f64>,
    pub(super) endogenous_report: String,
    pub(super) endogenous_summary: String,
    pub(super) stage_metrics: String,
}
