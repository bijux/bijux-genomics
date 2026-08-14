//! Owner: bijux-dna-bench-model
//! Persisted executable benchmark specification contracts.

use std::collections::BTreeMap;

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{de::Error as _, Deserialize, Deserializer, Serialize};

use crate::contract::{
    BENCHMARK_EFFECTIVE_PARAMETERS_SCHEMA_V1, BENCHMARK_EXECUTION_SPEC_SCHEMA_V1,
    BENCHMARK_STAGE_PLAN_MANIFEST_SCHEMA_V1,
};

use super::{BenchmarkAttemptIdentity, BenchmarkParameterValue};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkExecutionInputKind {
    Data,
    Reference,
    Database,
    UpstreamResult,
    ExpectedEvidence,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkExecutionSourceLock {
    #[schemars(schema_with = "repository_commit_schema")]
    pub repository_commit: String,
    pub repository_path: String,
    #[schemars(schema_with = "contract_sha256_schema")]
    pub contract_sha256: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkExecutionImage {
    pub path: String,
    #[schemars(schema_with = "sha256_identity_schema")]
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkExecutionInput {
    pub id: String,
    pub kind: BenchmarkExecutionInputKind,
    pub path: String,
    pub mount_path: String,
    #[schemars(schema_with = "sha256_identity_schema")]
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkExecutionCommands {
    pub version: Vec<String>,
    pub help: Vec<String>,
    pub warmup: Vec<String>,
    pub measured: Vec<String>,
    pub negative: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkExecutionParameterEvidence {
    pub parameter_schema_version: String,
    pub defaults_provider: String,
    pub overrides_path: String,
    #[schemars(schema_with = "sha256_identity_schema")]
    pub overrides_sha256: String,
    pub effective_path: String,
    #[schemars(schema_with = "sha256_identity_schema")]
    pub effective_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkEffectiveParameters {
    #[serde(deserialize_with = "deserialize_effective_parameters_schema_version")]
    #[schemars(schema_with = "effective_parameters_schema_version")]
    pub schema_version: String,
    pub stage_id: String,
    pub parameter_schema_version: String,
    pub defaults_provider: String,
    pub overrides: BTreeMap<String, BenchmarkParameterValue>,
    pub resolved_by_sample: BTreeMap<String, BTreeMap<String, BenchmarkEffectiveParameterValue>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(untagged)]
pub enum BenchmarkEffectiveParameterValue {
    Null,
    Boolean(bool),
    Integer(i64),
    Number(f64),
    Text(String),
    List(Vec<Self>),
    Object(BTreeMap<String, Self>),
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkExecutionDownstreamProbe {
    pub stage_id: String,
    pub input_contract: String,
    pub source: String,
    pub artifact: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkExecutionInterface {
    pub accepted_input_contracts: Vec<String>,
    pub output_contract: String,
    pub normalized_result_contract: String,
    pub adapter: String,
    pub downstream_probe: BenchmarkExecutionDownstreamProbe,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkExecutionMeasurement {
    pub warmup_repetitions: u32,
    pub measured_repetitions: u32,
    pub resource_file_template: String,
    pub runtime_scope: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkExecutionOutputs {
    pub result_root: String,
    pub mount_path: String,
    /// Required artifact paths in canonical order. An entry may resolve to one regular file or a
    /// nonempty directory whose complete regular-file tree is manifest-bound at completion.
    pub required_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkExecutionRetryPolicy {
    pub max_attempts: u32,
    pub backoff_seconds: u32,
    pub retryable_exit_codes: Vec<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkExecutionResources {
    pub account: String,
    pub partition: String,
    pub cpus: u32,
    pub memory_mb: u64,
    pub walltime: String,
    pub scratch_mb: u64,
    pub retry: BenchmarkExecutionRetryPolicy,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkExecutionCachePolicy {
    Cold,
    Warm,
    Disabled,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkExecutionDeterminism {
    Deterministic,
    Seeded,
    Nondeterministic,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkExecutionRuntimePolicy {
    pub locale: String,
    pub timezone: String,
    pub runtime_network: bool,
    pub threads: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub random_seed: Option<i64>,
    pub cache_policy: BenchmarkExecutionCachePolicy,
    pub determinism: BenchmarkExecutionDeterminism,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkAcceptanceOperator {
    Equal,
    GreaterOrEqual,
    GreaterThan,
    LessOrEqual,
    LessThan,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(untagged)]
pub enum BenchmarkAcceptanceExpectedValue {
    Boolean(bool),
    Integer(i64),
    Number(f64),
    Text(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkAcceptanceMetric {
    pub id: String,
    pub path: String,
    pub json_pointer: String,
    pub operator: BenchmarkAcceptanceOperator,
    pub expected: BenchmarkAcceptanceExpectedValue,
    pub unit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkExecutionAcceptance {
    pub version_regex: String,
    pub negative_exit_nonzero: bool,
    pub metrics: Vec<BenchmarkAcceptanceMetric>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkExecutionSpec {
    #[schemars(schema_with = "benchmark_execution_spec_schema_version")]
    pub schema_version: String,
    pub identity: BenchmarkAttemptIdentity,
    pub operator: String,
    pub prepared_at_utc: String,
    pub source: BenchmarkExecutionSourceLock,
    pub image: BenchmarkExecutionImage,
    pub inputs: Vec<BenchmarkExecutionInput>,
    pub commands: BenchmarkExecutionCommands,
    pub parameters: BenchmarkExecutionParameterEvidence,
    pub interface: BenchmarkExecutionInterface,
    pub measurement: BenchmarkExecutionMeasurement,
    pub outputs: BenchmarkExecutionOutputs,
    pub resources: BenchmarkExecutionResources,
    pub runtime: BenchmarkExecutionRuntimePolicy,
    pub acceptance: BenchmarkExecutionAcceptance,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkStagePlanReference {
    pub repetition: u32,
    pub execution_unit_id: String,
    pub stage_instance_id: String,
    pub path: String,
    #[schemars(schema_with = "sha256_identity_schema")]
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkStagePlanManifest {
    #[schemars(schema_with = "benchmark_stage_plan_manifest_schema_version")]
    pub schema_version: String,
    pub identity: BenchmarkAttemptIdentity,
    #[schemars(schema_with = "sha256_identity_schema")]
    pub execution_spec_sha256: String,
    pub plans: Vec<BenchmarkStagePlanReference>,
}

#[must_use]
pub fn benchmark_execution_spec_schema() -> Schema {
    schemars::schema_for!(BenchmarkExecutionSpec)
}

#[must_use]
pub fn benchmark_effective_parameters_schema() -> Schema {
    schemars::schema_for!(BenchmarkEffectiveParameters)
}

#[must_use]
pub fn benchmark_stage_plan_manifest_schema() -> Schema {
    schemars::schema_for!(BenchmarkStagePlanManifest)
}

fn benchmark_execution_spec_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": BENCHMARK_EXECUTION_SPEC_SCHEMA_V1,
    })
}

fn effective_parameters_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": BENCHMARK_EFFECTIVE_PARAMETERS_SCHEMA_V1,
    })
}

fn benchmark_stage_plan_manifest_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": BENCHMARK_STAGE_PLAN_MANIFEST_SCHEMA_V1,
    })
}

fn deserialize_effective_parameters_schema_version<'de, D>(
    deserializer: D,
) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    if value == BENCHMARK_EFFECTIVE_PARAMETERS_SCHEMA_V1 {
        Ok(value)
    } else {
        Err(D::Error::custom(format!("unsupported benchmark effective parameters: {value}")))
    }
}

fn sha256_identity_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "pattern": "^[0-9a-f]{64}$",
    })
}

fn repository_commit_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "pattern": "^[0-9a-f]{40}$",
    })
}

fn contract_sha256_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "object",
        "minProperties": 1,
        "propertyNames": {
            "pattern": "^[a-z0-9][a-z0-9_.-]*$"
        },
        "additionalProperties": {
            "type": "string",
            "pattern": "^[0-9a-f]{64}$"
        }
    })
}
