/// Expected-result identity and output declaration fields consumed by FASTQ proof evaluation.
/// Source acquisition retains ordering and multiplicity for owner admission.
#[derive(Debug, Clone)]
pub struct ExpectedResultObservation {
    pub domain: String,
    pub result_row_id: String,
    pub stage_id: String,
    pub tool_id: String,
    pub sample_scope: String,
    pub normalized_metrics_output_id: Option<String>,
    pub expected_output_artifact_ids: Vec<String>,
    pub stage_result_manifest_path: String,
    pub stdout_path: String,
    pub stderr_path: String,
    pub readiness_kind: String,
}
/// Complete canonical declaration status, mapped exhaustively at composition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComparableContract {
    Declared,
    MissingSharedMetrics,
}
/// Comparable metric declaration without importing benchmark-model serialization.
#[derive(Debug, Clone)]
pub struct ComparableMetricObservation {
    pub stage_id: String,
    pub comparison_contract_status: ComparableContract,
    pub shared_metric_fields: Vec<String>,
}
/// Structured command arguments remain separate strings with their original ordering.
#[derive(Debug, Clone)]
pub struct CommandArgvObservation {
    pub stage_id: String,
    pub tool_id: String,
    pub readiness_kind: String,
    pub argv: Vec<String>,
}
/// Address returned by the canonical schema publisher.
pub struct PublishedSchema {
    pub output_path: String,
}
