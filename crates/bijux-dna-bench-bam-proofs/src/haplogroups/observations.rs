// Native records retain their full persisted input shape, including unconsumed diagnostics.
use serde::Deserialize;
#[derive(Debug, Clone, Deserialize)]
pub(super) struct LocalHaplogroupsSmokeReport {
    pub(super) schema_version: String,
    pub(super) stage_id: String,
    pub(super) tool_id: String,
    pub(super) sample_id: String,
    pub(super) reference_panel_id: String,
    pub(super) reference_build: String,
    pub(super) case_count: usize,
    pub(super) rows: Vec<LocalHaplogroupsSmokeCaseReport>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code, reason = "retain native deserialization contract fields")]
pub(super) struct LocalHaplogroupsSmokeCaseReport {
    pub(super) schema_version: String,
    pub(super) stage_id: String,
    pub(super) tool_id: String,
    pub(super) proof_case: String,
    pub(super) sample_id: String,
    pub(super) expectation_matched: bool,
    pub(super) input_bam: String,
    pub(super) reference_fasta: String,
    pub(super) reference_panel: String,
    pub(super) reference_panel_id: String,
    pub(super) reference_build: String,
    pub(super) population_scope: String,
    pub(super) minimum_coverage: f64,
    pub(super) observed_mean_coverage: f64,
    pub(super) contamination_estimate: f64,
    pub(super) ready: bool,
    pub(super) haplogroup_call: Option<String>,
    pub(super) confidence: f64,
    pub(super) status: String,
    pub(super) markers_total: usize,
    pub(super) markers_supported: usize,
    pub(super) supported_marker_ids: Vec<String>,
    pub(super) refusal_codes: Vec<String>,
    pub(super) caveats: Vec<String>,
    pub(super) haplogroups_report: String,
    pub(super) haplogroups_summary: String,
    pub(super) haplogroup_report: String,
    pub(super) stage_metrics: String,
    pub(super) declared_output_ids: Vec<String>,
    pub(super) artifact_paths: Vec<String>,
}
