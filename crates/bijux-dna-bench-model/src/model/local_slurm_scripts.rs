//! Owner: bijux-dna-bench-model
//! Structured local SLURM script inventory produced by execution adapters.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const BENCHMARK_LOCAL_ALL_DOMAIN_SLURM_SCRIPTS_SCHEMA_V1: &str =
    "bijux.bench.local_all_domain_slurm_scripts.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalAllDomainSlurmScriptEntry {
    pub job_id_local: String,
    pub job_kind: String,
    pub result_id: Option<String>,
    pub pipeline_id: Option<String>,
    pub node_id: Option<String>,
    pub domain: String,
    pub stage_id: String,
    pub tool_id: String,
    pub corpus_id: String,
    pub asset_profile_id: String,
    pub dependency_count: usize,
    pub cpus_per_task: u32,
    pub memory_mb: u32,
    pub time_limit: String,
    pub script_path: String,
    pub stdout_path: String,
    pub stderr_path: String,
    pub command_step_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalAllDomainSlurmScriptsReport {
    pub schema_version: String,
    pub output_root: String,
    pub script_count: usize,
    pub benchmark_job_count: usize,
    pub essential_pipeline_job_count: usize,
    pub pipeline_count: usize,
    pub domain_counts: BTreeMap<String, usize>,
    pub benchmark_domain_counts: BTreeMap<String, usize>,
    pub essential_pipeline_domain_counts: BTreeMap<String, usize>,
    pub scripts: Vec<BenchmarkLocalAllDomainSlurmScriptEntry>,
}
