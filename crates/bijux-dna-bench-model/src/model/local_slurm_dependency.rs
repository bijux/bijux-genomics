//! Owner: bijux-dna-bench-model
//! Persisted evidence for Slurm dependency-source admission.

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use crate::contract::{
    BENCHMARK_LOCAL_SLURM_DEPENDENCY_CHECK_SCHEMA_V1, BENCHMARK_LOCAL_SLURM_SHELL_SYNTAX_SCHEMA_V1,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalSlurmDependencyCheckJob {
    pub job_name: String,
    pub stage_id: Option<String>,
    pub script_path: String,
    pub manifest_dependencies: Vec<String>,
    pub script_header_dependencies: Vec<String>,
    pub dependency_source: String,
    pub ok: bool,
    pub findings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalSlurmDependencyCheckReport {
    #[schemars(schema_with = "schema_version")]
    pub schema_version: String,
    pub root_path: String,
    pub manifest_path: String,
    pub report_path: String,
    pub job_count: usize,
    pub manifest_dependency_count: usize,
    pub script_header_dependency_count: usize,
    pub findings_count: usize,
    pub ok: bool,
    pub report_findings: Vec<String>,
    pub jobs: Vec<BenchmarkLocalSlurmDependencyCheckJob>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalSlurmShellSyntaxEntry {
    pub script_path: String,
    pub ok: bool,
    pub findings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalSlurmShellSyntaxReport {
    #[schemars(schema_with = "shell_syntax_schema_version")]
    pub schema_version: String,
    pub root_path: String,
    pub report_path: String,
    pub script_count: usize,
    pub findings_count: usize,
    pub ok: bool,
    pub report_findings: Vec<String>,
    pub scripts: Vec<BenchmarkLocalSlurmShellSyntaxEntry>,
}

#[must_use]
pub fn benchmark_local_slurm_dependency_check_schema() -> Schema {
    schemars::schema_for!(BenchmarkLocalSlurmDependencyCheckReport)
}

#[must_use]
pub fn benchmark_local_slurm_shell_syntax_schema() -> Schema {
    schemars::schema_for!(BenchmarkLocalSlurmShellSyntaxReport)
}

fn schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": BENCHMARK_LOCAL_SLURM_DEPENDENCY_CHECK_SCHEMA_V1
    })
}

fn shell_syntax_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": BENCHMARK_LOCAL_SLURM_SHELL_SYNTAX_SCHEMA_V1
    })
}
