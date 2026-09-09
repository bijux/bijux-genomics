//! Persisted output audit decisions and explicit observation inputs.

use std::collections::{BTreeMap, BTreeSet};

pub const OUTPUT_CONTRACT_TESTS_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.output_contract_audit.v1";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
// The persisted wire contract exposes each independent comparison as a boolean.
#[allow(clippy::struct_excessive_bools)]
pub struct OutputContractTestRow {
    pub result_id: String,
    pub domain: String,
    pub stage_id: String,
    pub tool_id: String,
    pub corpus_id: String,
    pub asset_profile_id: String,
    pub output_proof_surface: String,
    pub output_proof_path: String,
    pub proof_tool_id: Option<String>,
    pub runtime_path_proof_surface: String,
    pub runtime_path_proof_path: String,
    pub declared_raw_output_ids: Vec<String>,
    pub observed_raw_output_ids: Vec<String>,
    pub declared_normalized_metric_ids: Vec<String>,
    pub observed_normalized_metric_ids: Vec<String>,
    pub declared_index_output_ids: Vec<String>,
    pub observed_index_output_ids: Vec<String>,
    pub observed_output_paths: Vec<String>,
    pub stage_undeclared_output_ids: Vec<String>,
    pub declared_logs: Vec<String>,
    pub observed_logs: Vec<String>,
    pub declared_manifest: String,
    pub observed_manifest: String,
    pub raw_outputs_matched: bool,
    pub normalized_metrics_matched: bool,
    pub index_outputs_matched: bool,
    pub logs_matched: bool,
    pub manifest_matched: bool,
    pub no_undeclared_outputs: bool,
    pub independent_execution_proven: bool,
    pub execution_blocker: Option<String>,
    pub passed: bool,
    pub reason: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OutputContractTestsReport {
    pub schema_version: String,
    pub output_path: String,
    pub row_count: usize,
    pub passed_row_count: usize,
    pub blocked_row_count: usize,
    pub failed_row_count: usize,
    pub domain_counts: BTreeMap<String, usize>,
    pub output_proof_surface_counts: BTreeMap<String, usize>,
    pub blocked_stage_ids: Vec<String>,
    pub blocked_tool_ids: Vec<String>,
    pub failed_stage_ids: Vec<String>,
    pub failed_tool_ids: Vec<String>,
    pub rows: Vec<OutputContractTestRow>,
}

/// Observed output identities and execution attribution supplied by an adapter.
#[derive(Debug, Clone)]
pub struct OutputContractProof {
    pub output_proof_surface: String,
    pub output_proof_path: String,
    pub proof_tool_id: Option<String>,
    pub observed_ids: BTreeSet<String>,
    pub observed_paths_by_id: BTreeMap<String, Vec<String>>,
    pub stage_undeclared_output_ids: Vec<String>,
    pub independent_execution_proven: bool,
    pub execution_blocker: Option<String>,
}

/// Declared or observed runtime path projection; the proof surface retains its origin.
#[derive(Debug, Clone)]
pub struct OutputContractRuntimePaths {
    pub runtime_path_proof_surface: String,
    pub runtime_path_proof_path: String,
    pub stdout_path: String,
    pub stderr_path: String,
    pub stage_result_path: String,
}
