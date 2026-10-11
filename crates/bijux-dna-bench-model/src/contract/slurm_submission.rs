//! Owner: bijux-dna-bench-model
//! Semantic admission for persisted benchmark Slurm submission intents.

use std::collections::BTreeSet;

use anyhow::{bail, Result};
use chrono::DateTime;

use crate::model::slurm_submission::{
    BenchmarkSlurmDependencyCondition, BenchmarkSlurmSubmissionIntent,
};

use super::BENCHMARK_SLURM_SUBMISSION_INTENT_SCHEMA_V1;

/// Validates one persisted benchmark Slurm submission intent.
///
/// # Errors
///
/// Returns an error when its identity, dependency, digest, path, array, or timestamp evidence is
/// malformed.
pub fn validate_benchmark_slurm_submission_intent(
    intent: &BenchmarkSlurmSubmissionIntent,
) -> Result<()> {
    if intent.schema_version != BENCHMARK_SLURM_SUBMISSION_INTENT_SCHEMA_V1 {
        bail!("unsupported benchmark Slurm submission intent schema");
    }
    if intent.run_id.trim().is_empty() {
        bail!("benchmark Slurm submission intent has an empty run identity");
    }
    let mut dependencies = BTreeSet::new();
    if intent.dependencies.iter().any(|dependency| {
        dependency.is_empty()
            || !dependency.bytes().all(|byte| byte.is_ascii_digit())
            || !dependencies.insert(dependency)
    }) || (intent.dependencies.is_empty()
        && intent.dependency_condition == BenchmarkSlurmDependencyCondition::AfterAny)
    {
        bail!("benchmark Slurm submission intent has an invalid dependency identity");
    }
    for (name, digest) in [
        ("execution specification", intent.execution_spec_sha256.as_str()),
        ("execution-unit manifest", intent.stage_plan_manifest_sha256.as_str()),
        ("script", intent.script_sha256.as_str()),
    ] {
        if digest.len() != 64
            || !digest.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            bail!("benchmark Slurm submission intent has an invalid {name} digest");
        }
    }
    for (name, path) in [
        ("script", &intent.script_path),
        ("working directory", &intent.working_directory),
        ("stdout", &intent.stdout_path),
        ("stderr", &intent.stderr_path),
    ] {
        if path.as_os_str().is_empty() {
            bail!("benchmark Slurm submission intent has an empty {name} path");
        }
    }
    if let Some(array) = &intent.array {
        let task_count = array
            .last_task_id
            .checked_sub(array.first_task_id)
            .and_then(|difference| difference.checked_add(1));
        if task_count.is_none()
            || array.maximum_parallel_tasks.is_some_and(|limit| {
                limit == 0 || task_count.is_some_and(|task_count| limit > task_count)
            })
        {
            bail!("benchmark Slurm submission intent has an invalid array contract");
        }
    }
    DateTime::parse_from_rfc3339(&intent.submitted_at_utc)
        .map_err(|error| anyhow::anyhow!("invalid benchmark submission timestamp: {error}"))?;
    Ok(())
}
