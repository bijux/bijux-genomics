use serde::Deserialize;
#[derive(Debug, Clone, Deserialize)]
pub(super) struct LocalSexToolSmokeReport {
    pub(super) schema_version: String,
    pub(super) stage_id: String,
    pub(super) ready_sample_id: String,
    pub(super) insufficient_sample_id: String,
    pub(super) tool_ids: Vec<String>,
    pub(super) case_count: usize,
    pub(super) rows: Vec<LocalSexToolSmokeCaseReport>,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct LocalSexToolSmokeCaseReport {
    pub(super) schema_version: String,
    pub(super) stage_id: String,
    pub(super) tool_id: String,
    pub(super) proof_case: String,
    pub(super) sample_id: String,
    pub(super) expectation_matched: bool,
    pub(super) input_bam: String,
    pub(super) reference_fasta: String,
    pub(super) method: String,
    pub(super) chromosome_system: Option<String>,
    pub(super) minimum_y_sites: Option<u32>,
    pub(super) x_coverage: f64,
    pub(super) y_coverage: f64,
    pub(super) autosomal_coverage: f64,
    pub(super) x_to_y_ratio: Option<f64>,
    pub(super) call: bijux_dna_domain_bam::metrics::SexConfidenceClass,
    pub(super) confidence: f64,
    pub(super) status: String,
    pub(super) insufficiency_reason: Option<String>,
    pub(super) sex_report: String,
    pub(super) sex_estimate: String,
    pub(super) population_metrics: String,
    pub(super) haplogroup_report: String,
    pub(super) sex_summary: String,
    pub(super) stage_metrics: String,
    pub(super) declared_output_ids: Vec<String>,
    pub(super) artifact_paths: Vec<String>,
}
