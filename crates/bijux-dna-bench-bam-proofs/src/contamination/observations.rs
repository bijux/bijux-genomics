// Native records retain their full persisted input shape, including unconsumed diagnostics.
use serde::Deserialize;
#[derive(Debug, Clone, Deserialize)]
pub(super) struct LocalContaminationSmokeReport {
    pub(super) schema_version: String,
    pub(super) stage_id: String,
    pub(super) sample_id: String,
    pub(super) tool_ids: Vec<String>,
    pub(super) case_count: usize,
    pub(super) rows: Vec<LocalContaminationSmokeCaseReport>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code, reason = "retain native deserialization contract fields")]
pub(super) struct LocalContaminationSmokeCaseReport {
    pub(super) schema_version: String,
    pub(super) stage_id: String,
    pub(super) tool_id: String,
    pub(super) proof_case: String,
    pub(super) sample_id: String,
    pub(super) expectation_matched: bool,
    pub(super) input_bam: String,
    pub(super) reference_fasta: String,
    pub(super) scope: String,
    pub(super) tool_scope: String,
    pub(super) minimum_mean_coverage: f64,
    pub(super) prerequisites_passed: bool,
    pub(super) refusal_codes: Vec<String>,
    pub(super) caveats: Vec<String>,
    pub(super) raw_estimate: f64,
    pub(super) raw_ci_low: f64,
    pub(super) raw_ci_high: f64,
    pub(super) contamination_report: String,
    pub(super) contamination_summary: String,
    pub(super) stage_metrics: String,
    pub(super) declared_output_ids: Vec<String>,
    pub(super) artifact_paths: Vec<String>,
    pub(super) contamination_estimate: Option<String>,
    pub(super) contammix_report: Option<String>,
    pub(super) mt_consensus: Option<String>,
    pub(super) advisory_boundary: String,
    pub(super) contamination_modes: String,
    pub(super) contamination_stratified: String,
}
