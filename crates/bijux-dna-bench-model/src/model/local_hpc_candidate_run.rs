//! Persisted local HPC candidate-run contracts.

use std::collections::BTreeMap;

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use crate::{
    BenchmarkLocalHpcAssetStagingInput, BENCHMARK_LOCAL_HPC_CANDIDATE_RUN_MANIFEST_SCHEMA_V1,
};

pub const BENCHMARK_LOCAL_HPC_CANDIDATE_PROFILE_ID: &str = "small_runtime_surface_representatives";
pub const BENCHMARK_LOCAL_HPC_CANDIDATE_CPUS_PER_TASK_MAX: u32 = 4;
pub const BENCHMARK_LOCAL_HPC_CANDIDATE_MEMORY_MB_MAX: u32 = 4096;
pub const BENCHMARK_LOCAL_HPC_CANDIDATE_WALLTIME_MINUTES_MAX: u32 = 15;
pub const BENCHMARK_LOCAL_HPC_CANDIDATE_SCRATCH_GB_MAX: u32 = 2;
pub const BENCHMARK_LOCAL_HPC_CANDIDATE_STAGED_INPUT_BYTES_MAX: u64 = 10 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalHpcCandidateResourceCeiling {
    pub cpus_per_task_max: u32,
    pub memory_mb_max: u32,
    pub walltime_minutes_max: u32,
    pub scratch_gb_max: u32,
    pub staged_input_bytes_max: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[allow(clippy::struct_excessive_bools)]
pub struct BenchmarkLocalHpcCandidateBehavior {
    pub only_benchmark_result_jobs: bool,
    pub only_dependency_free_jobs: bool,
    pub only_known_assets: bool,
    pub only_known_execution_modes: bool,
    pub only_available_execution_resolution: bool,
    pub only_small_resource_envelopes: bool,
    pub only_small_input_footprints: bool,
    pub selected_rows_keep_expected_outputs: bool,
    pub selected_rows_keep_stop_conditions: bool,
    pub vcf_rows_excluded_for_unknown_execution_modes: bool,
    pub proven: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalHpcCandidateStopCondition {
    pub condition_id: String,
    pub source_surface: String,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalHpcCandidateResourceHint {
    pub cpus_per_task: u32,
    pub memory_mb: u32,
    pub time_limit: String,
    pub walltime_minutes: u32,
    pub scratch_gb: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalHpcCandidateJob {
    pub representative_group_id: String,
    pub selection_reason: String,
    pub job_id_local: String,
    pub result_id: String,
    pub domain: String,
    pub stage_id: String,
    pub tool_id: String,
    pub corpus_id: String,
    pub asset_profile_id: String,
    pub execution_mode: String,
    pub execution_mode_summary: String,
    pub resolution_kind: String,
    pub resolution_target: String,
    pub command_entrypoint: Option<String>,
    pub expected_stage_result_manifest_path: String,
    pub expected_outputs: Vec<String>,
    pub resource_hint: BenchmarkLocalHpcCandidateResourceHint,
    pub staged_input_count: usize,
    pub staged_input_total_bytes: u64,
    pub staged_inputs: Vec<BenchmarkLocalHpcAssetStagingInput>,
    pub stop_conditions: Vec<BenchmarkLocalHpcCandidateStopCondition>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalHpcCandidateRunManifest {
    #[schemars(schema_with = "schema_version")]
    pub schema_version: String,
    pub output_path: String,
    pub selection_profile_id: String,
    pub evaluated_job_count: usize,
    pub benchmark_job_count: usize,
    pub root_benchmark_job_count: usize,
    pub eligible_small_job_count: usize,
    pub representative_group_count: usize,
    pub selected_job_count: usize,
    pub selected_staged_input_count: usize,
    pub selected_staged_input_bytes: u64,
    pub selected_domain_counts: BTreeMap<String, usize>,
    pub selected_execution_mode_counts: BTreeMap<String, usize>,
    pub selected_resolution_kind_counts: BTreeMap<String, usize>,
    pub exclusion_reason_counts: BTreeMap<String, usize>,
    pub resource_ceiling: BenchmarkLocalHpcCandidateResourceCeiling,
    pub behavior: BenchmarkLocalHpcCandidateBehavior,
    pub rows: Vec<BenchmarkLocalHpcCandidateJob>,
}

#[must_use]
pub fn benchmark_local_hpc_candidate_run_manifest_schema() -> Schema {
    schemars::schema_for!(BenchmarkLocalHpcCandidateRunManifest)
}

fn schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": BENCHMARK_LOCAL_HPC_CANDIDATE_RUN_MANIFEST_SCHEMA_V1,
    })
}
