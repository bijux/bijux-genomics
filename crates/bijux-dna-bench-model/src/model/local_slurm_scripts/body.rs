//! Owner: bijux-dna-bench-model
//! Persisted script-text inspection evidence; not proof of shell execution.
use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

pub const BENCHMARK_LOCAL_SLURM_SCRIPT_BODY_SCHEMA_V1: &str =
    "bijux.bench.local_slurm_script_bodies.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalSlurmScriptBodyReport {
    #[schemars(schema_with = "schema_version")]
    pub schema_version: String,
    pub root_path: String,
    pub report_path: String,
    pub script_count: usize,
    pub findings_count: usize,
    pub ok: bool,
    pub report_findings: Vec<String>,
    pub scripts: Vec<BenchmarkLocalSlurmScriptBodyEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalSlurmScriptBodyEntry {
    pub script_path: String,
    pub has_bijux_dna_command: bool,
    pub findings: Vec<String>,
    pub ok: bool,
}

fn schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type":"string","const":BENCHMARK_LOCAL_SLURM_SCRIPT_BODY_SCHEMA_V1})
}
