use serde::Deserialize;
#[derive(Debug, Clone, Deserialize)]
pub(super) struct LocalRecalibrationSmokeReport {
    pub(super) schema_version: String,
    pub(super) stage_id: String,
    pub(super) tool_id: String,
    pub(super) sample_id: String,
    pub(super) expectation_matched: bool,
    pub(super) input_bam: String,
    pub(super) reference_fasta: String,
    pub(super) known_sites: Vec<String>,
    pub(super) known_sites_asset_ids: Vec<String>,
    pub(super) requested_mode: String,
    pub(super) effective_mode: String,
    pub(super) status: String,
    pub(super) reason: String,
    pub(super) coverage_gate: LocalRecalibrationCoverageGate,
    pub(super) observed_mean_coverage: f64,
    pub(super) observed_breadth_1x: f64,
    pub(super) output_bam_present: bool,
    pub(super) recalibration_report_present: bool,
    pub(super) recalibrated_bam: String,
    pub(super) recalibration_report: String,
    pub(super) recalibration_summary: String,
    pub(super) stage_metrics: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct LocalRecalibrationCoverageGate {
    pub(super) min_mean_coverage: f64,
    pub(super) min_breadth_1x: f64,
}
