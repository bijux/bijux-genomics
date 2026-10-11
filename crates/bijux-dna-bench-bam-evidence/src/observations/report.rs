use std::collections::BTreeMap;
/// Report identity fields consumed from the shared expected-result schema.
#[derive(Debug, Clone)]
pub struct ExpectedResultBinding {
    pub result_row_id: String,
    pub domain: String,
    pub stage_id: String,
    pub tool_id: String,
    pub corpus_family_id: String,
    pub fixture_id: String,
    pub sample_scope: String,
}
/// Tool support observed from the governed admission catalog.
#[derive(Debug, Clone)]
pub struct ToolAdmission {
    pub tool_id: String,
    pub support_status: String,
}
pub type StageAdmissions = BTreeMap<String, Vec<ToolAdmission>>;
/// Inventory stage identity and its canonical readiness label.
#[derive(Debug, Clone)]
pub struct StageReadiness {
    pub stage_id: String,
    pub readiness_kind: &'static str,
}
/// Domain/planner facts used for report placement. The evidence owner does not
/// maintain another canonical stage registry or select executable tools.
#[derive(Debug, Clone)]
pub struct ReportStage {
    pub canonical_rank: Option<usize>,
    pub criticality: Option<&'static str>,
    pub preferred_tool_id: String,
    pub workflow_branch_id: Option<String>,
    pub scientific_context_required: Vec<String>,
}
