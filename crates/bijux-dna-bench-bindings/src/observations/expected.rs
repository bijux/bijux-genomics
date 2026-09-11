/// The canonical output owner decides completeness; this is not another wire schema.
#[derive(Debug, Clone)]
pub struct OutputContract {
    pub stage_id: String,
    pub tool_id: String,
    pub complete: bool,
    pub execution_expected_output_ids: Vec<String>,
    pub raw_output_artifact_ids: Vec<String>,
    pub normalized_metrics_output_id: Option<String>,
}
/// Corpus assignment identity, before required-field admission.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssignmentKind {
    Assigned,
    AssetBacked,
    Excluded,
}
/// Consumed canonical assignment fields; optional values retain malformed-source evidence.
#[derive(Debug, Clone)]
pub struct CorpusAssignment {
    pub stage_id: String,
    pub tool_id: String,
    pub benchmark_status: String,
    pub assignment_status: AssignmentKind,
    pub corpus_family_id: Option<String>,
    pub fixture_id: Option<String>,
    pub benchmark_scope_id: Option<String>,
}
