//! Persisted local HPC resume-simulation contracts.

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use crate::BENCHMARK_LOCAL_HPC_RESUME_SIMULATION_SCHEMA_V1;

pub const BENCHMARK_LOCAL_HPC_RESUME_FAILED_JOB_ID: &str =
    "pipeline:relatedness-segments-vcf:vcf.ibd";
pub const BENCHMARK_LOCAL_HPC_RESUME_DEPENDENCY_JOB_ID: &str =
    "pipeline:relatedness-segments-vcf:vcf.demography";
pub const BENCHMARK_LOCAL_HPC_RESUME_UNRELATED_JOB_ID: &str =
    "pipeline:relatedness-segments-vcf:vcf.roh";
pub const BENCHMARK_LOCAL_HPC_RESUME_MISSING_MANIFEST_JOB_ID: &str =
    "benchmark:vcf:vcf_production_regression:vcf.stats:vcf_cohort:bcftools";
pub const BENCHMARK_LOCAL_HPC_RESUME_STALE_OUTPUT_JOB_ID: &str =
    "benchmark:bam:corpus-01-adna-bam-mini:bam.contamination:sample-set:contammix";
pub const BENCHMARK_LOCAL_HPC_RESUME_VALID_JOB_ID: &str =
    "benchmark:vcf:vcf_production_regression:vcf.qc:vcf_cohort:bcftools";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[allow(clippy::struct_excessive_bools)]
pub struct BenchmarkLocalHpcResumeSimulationBehavior {
    pub failed_stage_result_job_id: String,
    pub dependency_rerun_job_id: String,
    pub unrelated_valid_job_id: String,
    pub missing_manifest_job_id: String,
    pub stale_partial_output_job_id: String,
    pub valid_completed_job_id: String,
    pub failed_stage_result_reruns: bool,
    pub dependency_rerun_propagates: bool,
    pub unrelated_valid_job_skips: bool,
    pub missing_manifest_reruns: bool,
    pub stale_partial_output_reruns: bool,
    pub valid_completed_job_skips: bool,
    pub proven: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalHpcResumeSimulationJob {
    pub job_id_local: String,
    pub job_kind: String,
    pub result_id: Option<String>,
    pub pipeline_id: Option<String>,
    pub node_id: Option<String>,
    pub stage_id: String,
    pub tool_id: String,
    pub dependency_count: usize,
    pub dependencies: Vec<String>,
    pub completion_state: String,
    pub completion_detail: String,
    pub manifest_valid: bool,
    pub runtime_status: Option<String>,
    pub declared_output_count: usize,
    pub present_output_count: usize,
    pub missing_output_paths: Vec<String>,
    pub resume_action: String,
    pub reason: String,
    pub rerun_dependencies: Vec<String>,
    pub result_root: String,
    pub stage_result_manifest_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalHpcResumeSimulationReport {
    #[schemars(schema_with = "schema_version")]
    pub schema_version: String,
    pub manifest_path: String,
    pub output_path: String,
    pub simulation_root: String,
    pub job_count: usize,
    pub benchmark_job_count: usize,
    pub essential_pipeline_job_count: usize,
    pub dependency_count: usize,
    pub valid_completed_job_count: usize,
    pub failed_stage_result_job_count: usize,
    pub invalid_stage_result_job_count: usize,
    pub missing_stage_result_job_count: usize,
    pub stale_partial_output_job_count: usize,
    pub skip_job_count: usize,
    pub rerun_job_count: usize,
    pub dependency_rerun_job_count: usize,
    pub behavior: BenchmarkLocalHpcResumeSimulationBehavior,
    pub jobs: Vec<BenchmarkLocalHpcResumeSimulationJob>,
}

#[must_use]
pub fn benchmark_local_hpc_resume_simulation_schema() -> Schema {
    schemars::schema_for!(BenchmarkLocalHpcResumeSimulationReport)
}

fn schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type":"string","const":BENCHMARK_LOCAL_HPC_RESUME_SIMULATION_SCHEMA_V1})
}
