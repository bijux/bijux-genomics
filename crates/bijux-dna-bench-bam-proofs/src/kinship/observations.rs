use serde::Deserialize;
#[derive(Debug, Clone, Deserialize)]
pub(super) struct LocalKinshipSmokeReport {
    pub(super) schema_version: String,
    pub(super) stage_id: String,
    pub(super) case_count: usize,
    pub(super) all_cases_matched: bool,
    pub(super) cases: Vec<LocalKinshipSmokeCaseReport>,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct LocalKinshipSmokeCaseReport {
    pub(super) sample_id: String,
    pub(super) expectation_matched: bool,
    #[allow(dead_code, reason = "retain required native deserialization field")]
    pub(super) input_bam: String,
    pub(super) method: String,
    pub(super) reference_panel: String,
    pub(super) reference_build: String,
    pub(super) population_scope: String,
    pub(super) min_overlap_snps: u64,
    pub(super) requires_cohort_context: bool,
    pub(super) observed_max_overlap_snps: u64,
    pub(super) pair_count: u64,
    pub(super) status: String,
    pub(super) insufficiency_reason: Option<String>,
    pub(super) pairwise_results: Vec<LocalKinshipPairwiseResult>,
    pub(super) kinship_report: String,
    pub(super) kinship_summary: String,
    pub(super) kinship_segments: String,
    pub(super) stage_metrics: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct LocalKinshipPairwiseResult {
    pub(super) sample_a: String,
    pub(super) sample_b: String,
    pub(super) overlap_snps: u64,
    pub(super) kinship_coefficient: f64,
    pub(super) relationship_label: String,
}
