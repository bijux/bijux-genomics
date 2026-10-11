//! Owner: bijux-dna-bench-model
//! Persisted benchmark Slurm submission intent records.

use std::path::PathBuf;

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use crate::contract::BENCHMARK_SLURM_SUBMISSION_INTENT_SCHEMA_V1;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkSlurmDependencyCondition {
    #[default]
    AfterOk,
    AfterAny,
}

impl BenchmarkSlurmDependencyCondition {
    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn is_after_ok(value: &Self) -> bool {
        matches!(value, Self::AfterOk)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkSlurmArraySpec {
    pub first_task_id: u32,
    pub last_task_id: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub maximum_parallel_tasks: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkSlurmSubmissionIntent {
    #[schemars(schema_with = "submission_intent_schema_version")]
    pub schema_version: String,
    pub run_id: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<String>,
    #[serde(default, skip_serializing_if = "BenchmarkSlurmDependencyCondition::is_after_ok")]
    pub dependency_condition: BenchmarkSlurmDependencyCondition,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub array: Option<BenchmarkSlurmArraySpec>,
    #[schemars(schema_with = "sha256_identity_schema")]
    pub execution_spec_sha256: String,
    #[schemars(schema_with = "sha256_identity_schema")]
    pub stage_plan_manifest_sha256: String,
    #[schemars(schema_with = "sha256_identity_schema")]
    pub script_sha256: String,
    pub script_path: PathBuf,
    pub working_directory: PathBuf,
    pub stdout_path: PathBuf,
    pub stderr_path: PathBuf,
    pub submitted_at_utc: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchmarkSlurmSubmissionMaterial {
    pub schema_version: String,
    pub run_id: String,
    pub dependencies: Vec<String>,
    pub dependency_condition: BenchmarkSlurmDependencyCondition,
    pub array: Option<BenchmarkSlurmArraySpec>,
    pub execution_spec_sha256: String,
    pub execution_unit_manifest_sha256: String,
    pub script_sha256: String,
    pub script_path: PathBuf,
    pub working_directory: PathBuf,
    pub stdout_path: PathBuf,
    pub stderr_path: PathBuf,
    pub submitted_at_utc: String,
}

impl BenchmarkSlurmSubmissionIntent {
    #[must_use]
    pub fn from_material(material: BenchmarkSlurmSubmissionMaterial) -> Self {
        Self {
            schema_version: material.schema_version,
            run_id: material.run_id,
            dependencies: material.dependencies,
            dependency_condition: material.dependency_condition,
            array: material.array,
            execution_spec_sha256: material.execution_spec_sha256,
            stage_plan_manifest_sha256: material.execution_unit_manifest_sha256,
            script_sha256: material.script_sha256,
            script_path: material.script_path,
            working_directory: material.working_directory,
            stdout_path: material.stdout_path,
            stderr_path: material.stderr_path,
            submitted_at_utc: material.submitted_at_utc,
        }
    }

    #[must_use]
    pub fn material(&self) -> BenchmarkSlurmSubmissionMaterial {
        BenchmarkSlurmSubmissionMaterial {
            schema_version: self.schema_version.clone(),
            run_id: self.run_id.clone(),
            dependencies: self.dependencies.clone(),
            dependency_condition: self.dependency_condition,
            array: self.array.clone(),
            execution_spec_sha256: self.execution_spec_sha256.clone(),
            execution_unit_manifest_sha256: self.stage_plan_manifest_sha256.clone(),
            script_sha256: self.script_sha256.clone(),
            script_path: self.script_path.clone(),
            working_directory: self.working_directory.clone(),
            stdout_path: self.stdout_path.clone(),
            stderr_path: self.stderr_path.clone(),
            submitted_at_utc: self.submitted_at_utc.clone(),
        }
    }
}

#[must_use]
pub fn benchmark_slurm_submission_intent_schema() -> Schema {
    schemars::schema_for!(BenchmarkSlurmSubmissionIntent)
}

fn submission_intent_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": BENCHMARK_SLURM_SUBMISSION_INTENT_SCHEMA_V1,
    })
}

fn sha256_identity_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "pattern": "^[0-9a-f]{64}$",
    })
}
