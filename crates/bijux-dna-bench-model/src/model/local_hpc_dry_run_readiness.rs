//! Persisted local HPC dry-run readiness contracts.

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use crate::BENCHMARK_LOCAL_HPC_DRY_RUN_READINESS_SCHEMA_V1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalHpcDryRunReadinessGoalCheck {
    pub goal_id: u32,
    pub surface: String,
    pub output_path: Option<String>,
    pub ok: bool,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalHpcDryRunReadinessSummary {
    pub benchmark_job_count: usize,
    pub essential_pipeline_job_count: usize,
    pub staged_input_count: usize,
    pub execution_resolver_row_count: usize,
    pub execution_unclassified_tool_count: usize,
    pub stage_benchmark_row_count: usize,
    pub pipeline_node_row_count: usize,
    pub dependency_case_count: usize,
    pub resume_rerun_job_count: usize,
    pub result_collection_row_count: usize,
    pub candidate_job_count: usize,
    pub candidate_representative_group_count: usize,
    pub candidate_selected_input_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[allow(clippy::struct_excessive_bools)]
pub struct BenchmarkLocalHpcDryRunReadinessBehavior {
    pub goals_481_to_489_validated: bool,
    pub benchmark_jobs_keep_known_assets: bool,
    pub benchmark_jobs_keep_scratch_layout: bool,
    pub benchmark_jobs_keep_explicit_execution_resolution: bool,
    pub pipeline_nodes_keep_dependency_manifests: bool,
    pub dependency_failures_block_descendants_only: bool,
    pub resume_rules_are_explicit: bool,
    pub result_collection_statuses_are_distinct: bool,
    pub first_run_candidate_is_small_and_governed: bool,
    pub no_manual_steps_or_ambiguity: bool,
    pub proven: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalHpcDryRunReadinessReport {
    #[schemars(schema_with = "schema_version")]
    pub schema_version: String,
    pub output_path: String,
    pub checked_goal_count: usize,
    pub passed_goal_count: usize,
    pub failed_goal_count: usize,
    pub failing_goal_ids: Vec<u32>,
    pub ready_for_first_hpc_run: bool,
    pub summary: BenchmarkLocalHpcDryRunReadinessSummary,
    pub behavior: BenchmarkLocalHpcDryRunReadinessBehavior,
    pub checks: Vec<BenchmarkLocalHpcDryRunReadinessGoalCheck>,
}

#[must_use]
pub fn benchmark_local_hpc_dry_run_readiness_schema() -> Schema {
    schemars::schema_for!(BenchmarkLocalHpcDryRunReadinessReport)
}

fn schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type":"string","const":BENCHMARK_LOCAL_HPC_DRY_RUN_READINESS_SCHEMA_V1})
}
