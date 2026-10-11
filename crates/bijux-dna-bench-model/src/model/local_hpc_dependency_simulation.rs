//! Persisted local HPC dependency-simulation contracts.

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use crate::BENCHMARK_LOCAL_HPC_DEPENDENCY_SIMULATION_SCHEMA_V1;

pub const BENCHMARK_LOCAL_HPC_RELATEDNESS_FAILURE_JOB_ID: &str =
    "pipeline:relatedness-segments-vcf:vcf.ibd";
pub const BENCHMARK_LOCAL_HPC_RELATEDNESS_BLOCKED_JOB_ID: &str =
    "pipeline:relatedness-segments-vcf:vcf.demography";
pub const BENCHMARK_LOCAL_HPC_RELATEDNESS_CONTINUED_JOB_ID: &str =
    "pipeline:relatedness-segments-vcf:vcf.roh";
pub const BENCHMARK_LOCAL_HPC_REFERENCE_FAILURE_JOB_ID: &str =
    "pipeline:reference-panel-imputation:vcf.prepare_reference_panel";
pub const BENCHMARK_LOCAL_HPC_REFERENCE_BLOCKED_JOB_ID: &str =
    "pipeline:reference-panel-imputation:vcf.phasing";
pub const BENCHMARK_LOCAL_HPC_REFERENCE_CONTINUED_JOB_ID: &str =
    "pipeline:reference-panel-imputation:vcf.qc";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalHpcDependencySimulationJob {
    pub job_id_local: String,
    pub job_kind: String,
    pub result_id: Option<String>,
    pub pipeline_id: Option<String>,
    pub node_id: Option<String>,
    pub stage_id: String,
    pub dependency_count: usize,
    pub dependencies: Vec<String>,
    pub dependent_count: usize,
    pub dependents: Vec<String>,
    pub blocking_dependencies: Vec<String>,
    pub status: String,
    pub descendant_of_failed_job: bool,
    pub unrelated_to_failed_job: bool,
    pub start_second: u64,
    pub finish_second: u64,
    pub duration_seconds: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[allow(clippy::struct_excessive_bools)]
pub struct BenchmarkLocalHpcDependencySimulationCase {
    pub case_id: String,
    pub failed_job_id: String,
    pub blocked_descendant_job_id: String,
    pub continued_sibling_job_id: String,
    pub descendant_job_ids: Vec<String>,
    pub blocked_job_ids: Vec<String>,
    pub continued_unrelated_job_ids: Vec<String>,
    pub continued_benchmark_job_ids: Vec<String>,
    pub simulated_makespan_seconds: u64,
    pub blocked_only_descendants: bool,
    pub unrelated_branches_continue: bool,
    pub benchmark_jobs_continue: bool,
    pub proven: bool,
    pub jobs: Vec<BenchmarkLocalHpcDependencySimulationJob>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalHpcDependencySimulationReport {
    #[schemars(schema_with = "schema_version")]
    pub schema_version: String,
    pub manifest_path: String,
    pub output_path: String,
    pub job_count: usize,
    pub benchmark_job_count: usize,
    pub essential_pipeline_job_count: usize,
    pub dependency_count: usize,
    pub case_count: usize,
    pub all_cases_proven: bool,
    pub cases: Vec<BenchmarkLocalHpcDependencySimulationCase>,
}

#[must_use]
pub fn benchmark_local_hpc_dependency_simulation_schema() -> Schema {
    schemars::schema_for!(BenchmarkLocalHpcDependencySimulationReport)
}

fn schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type":"string","const":BENCHMARK_LOCAL_HPC_DEPENDENCY_SIMULATION_SCHEMA_V1})
}
