/// Consumed fields of a canonical serving-map row; not a persisted schema.
#[derive(Debug, Clone)]
pub struct ServingBinding {
    pub tool_id: String,
    pub stage_id: String,
    pub support_status: String,
    pub adapter_status: String,
    pub parser_status: String,
    pub corpus_status: String,
}
/// Canonical inventory cardinalities and serving observations.
#[derive(Debug, Clone)]
pub struct ServingInventory {
    pub stage_count: usize,
    pub tool_count: usize,
    pub rows: Vec<ServingBinding>,
}
/// Planner-owned output declarations and assessment, observed together.
#[derive(Debug, Clone)]
pub struct OutputAssessment {
    /// Completion computed by the canonical planner assessment.
    pub complete: bool,
    pub declared_output_ids: Vec<String>,
    pub execution_expected_output_ids: Vec<String>,
    pub stage_expected_artifact_ids: Vec<String>,
    pub normalized_metrics_output_id: Option<String>,
    pub raw_output_artifact_ids: Vec<String>,
    pub missing_declarations: Vec<String>,
}
