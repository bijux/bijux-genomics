use serde::Serialize;
use std::collections::BTreeMap;
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ExpectedBenchmarkResultRow {
    pub result_row_id: String,
    pub domain: String,
    pub stage_id: String,
    pub tool_id: String,
    pub readiness_kind: String,
    pub corpus_family_id: String,
    pub fixture_id: String,
    pub sample_scope: String,
    pub expected_output_artifact_ids: Vec<String>,
    pub raw_output_artifact_ids: Vec<String>,
    pub normalized_metrics_output_id: Option<String>,
    pub result_root: String,
    pub stage_result_manifest_path: String,
    pub stdout_path: String,
    pub stderr_path: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExpectedBenchmarkResultsReport {
    pub schema_version: &'static str,
    pub output_path: String,
    pub stage_count: usize,
    pub tool_count: usize,
    pub row_count: usize,
    pub domain_counts: BTreeMap<String, usize>,
    pub fixture_counts: BTreeMap<String, usize>,
    pub rows: Vec<ExpectedBenchmarkResultRow>,
}
