use std::path::PathBuf;

use serde::{Deserialize, Serialize};

pub const CONTAINER_PIPELINE_SCHEMA_V1: &str = "bijux.container.pipeline.v1";
pub const CONTAINER_PIPELINE_SCHEMA_V2: &str = "bijux.container.pipeline.v2";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContainerPipelineV1 {
    pub schema_version: String,
    pub run: PipelineRunV1,
    pub repository: PipelineRepository,
    pub image: PipelineImage,
    pub data: PipelineDataV1,
    pub commands: PipelineCommandsV1,
    pub outputs: PipelineOutputsV1,
    pub resources: PipelineResourcesV1,
    pub execution: PipelineExecutionV1,
    pub acceptance: PipelineAcceptanceV1,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContainerPipelineV2 {
    pub schema_version: String,
    pub run: PipelineRunV2,
    pub repository: PipelineRepository,
    pub image: PipelineImage,
    pub data: PipelineDataV2,
    pub commands: PipelineCommandsV2,
    pub outputs: PipelineOutputsV2,
    pub resources: PipelineResourcesV2,
    pub execution: PipelineExecutionV2,
    pub dependencies: Vec<PipelineDependency>,
    pub acceptance: PipelineAcceptanceV2,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parameters: Option<PipelineParameterEvidence>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interface: Option<PipelineInterfaceEvidence>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub measurement: Option<PipelineMeasurement>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ContainerPipelineContract {
    Legacy(Box<ContainerPipelineV1>),
    Current(Box<ContainerPipelineV2>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PipelineRunV1 {
    pub id: String,
    pub campaign: String,
    pub corpus: String,
    pub sample: String,
    pub domain: String,
    pub stage: String,
    pub tool: String,
    pub operator: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PipelineRunV2 {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<PipelineEvidenceKind>,
    pub campaign: String,
    pub corpus: String,
    pub sample: String,
    pub domain: String,
    pub stage: String,
    pub tool: String,
    pub operator: String,
    pub submitted_at_utc: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PipelineEvidenceKind {
    Qualification,
    Benchmark,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PipelineRepository {
    pub path: PathBuf,
    pub commit: String,
    pub clean: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PipelineImage {
    pub path: PathBuf,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PipelineDataV1 {
    pub inputs: Vec<PipelineInputV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PipelineInputV1 {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<PipelineInputKind>,
    pub path: PathBuf,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PipelineDataV2 {
    pub inputs: Vec<PipelineInputV2>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PipelineInputV2 {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<PipelineInputKind>,
    pub path: PathBuf,
    pub mount_path: PathBuf,
    pub sha256: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PipelineInputKind {
    Data,
    Reference,
    Database,
    UpstreamResult,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PipelineCommandsV1 {
    pub version: Vec<String>,
    pub help: Vec<String>,
    pub real: Vec<String>,
    pub negative: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PipelineCommandsV2 {
    pub version: Vec<String>,
    pub help: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub warmup: Option<Vec<String>>,
    pub real: Vec<String>,
    pub negative: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PipelineOutputsV1 {
    pub result_root: PathBuf,
    pub required_files: Vec<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PipelineOutputsV2 {
    pub result_root: PathBuf,
    pub mount_path: PathBuf,
    pub required_files: Vec<PathBuf>,
    pub collection_rule: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PipelineResourcesV1 {
    pub account: String,
    pub partition: String,
    pub cpus: u64,
    pub memory_mb: u64,
    pub walltime: String,
    pub scratch_mb: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PipelineResourcesV2 {
    pub account: String,
    pub partition: String,
    pub cpus: u64,
    pub memory_mb: u64,
    pub walltime: String,
    pub scratch_mb: u64,
    pub retry: PipelineRetry,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PipelineRetry {
    pub max_attempts: u64,
    pub backoff_seconds: u64,
    pub retryable_exit_codes: Vec<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PipelineExecutionV1 {
    pub locale: String,
    pub timezone: String,
    pub runtime_network: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PipelineExecutionV2 {
    pub locale: String,
    pub timezone: String,
    pub runtime_network: bool,
    pub threads: u64,
    pub random_seed: Option<i64>,
    pub cache_policy: PipelineCachePolicy,
    pub determinism: PipelineDeterminism,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PipelineCachePolicy {
    Cold,
    Warm,
    Disabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PipelineDeterminism {
    Deterministic,
    Seeded,
    Nondeterministic,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PipelineAcceptanceV1 {
    pub version_regex: String,
    pub negative_exit_nonzero: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PipelineAcceptanceV2 {
    pub version_regex: String,
    pub negative_exit_nonzero: bool,
    pub metrics: Vec<PipelineAcceptanceMetric>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PipelineAcceptanceMetric {
    pub id: String,
    pub path: PathBuf,
    pub json_pointer: String,
    pub operator: PipelineMetricOperator,
    pub expected: serde_json::Value,
    pub unit: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PipelineMetricOperator {
    Eq,
    Ge,
    Gt,
    Le,
    Lt,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PipelineDependency {
    pub run_id: String,
    pub slurm_job_id: String,
    pub required_outputs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PipelineMeasurement {
    pub schema_version: String,
    pub warmup_repetitions: u64,
    pub measured_repetitions: u64,
    pub resource_file_template: String,
    pub runtime_scope: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PipelineParameterEvidence {
    pub schema_version: String,
    pub parameter_schema_version: String,
    pub defaults_provider: String,
    pub overrides_file: PathBuf,
    pub overrides_sha256: String,
    pub effective_file: PathBuf,
    pub effective_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PipelineInterfaceEvidence {
    pub schema_version: String,
    pub accepted_input_contracts: Vec<String>,
    pub output_contract: String,
    pub normalized_result_contract: PathBuf,
    pub adapter: String,
    pub downstream_probe: PipelineDownstreamProbe,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PipelineDownstreamProbe {
    pub stage_id: String,
    pub input_contract: String,
    pub source: String,
    pub artifact: PathBuf,
}
