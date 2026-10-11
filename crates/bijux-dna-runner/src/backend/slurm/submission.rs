//! Owner: bijux-dna-runner
//! Crash-bounded submission of prepared immutable benchmark jobs.

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{
    validate_benchmark_slurm_submission_intent, validate_execution_spec, BenchmarkExecutionSpec,
    BenchmarkSlurmArraySpec, BenchmarkSlurmDependencyCondition, BenchmarkSlurmSubmissionIntent,
    BenchmarkSlurmSubmissionMaterial, BENCHMARK_SLURM_SUBMISSION_INTENT_SCHEMA_V1,
};
use serde::{Deserialize, Serialize};

use super::{
    submit_slurm_job, PreparedBenchmarkSlurmJob, SlurmArraySpec, SlurmClientPaths,
    SlurmDependencyCondition, SlurmJobHandle, SLURM_JOB_HANDLE_SCHEMA_V1,
};

const SUBMISSION_FAILURE_SCHEMA_V1: &str = "bijux.runner.benchmark_submission_failure.v1";
const MAX_EVIDENCE_BYTES: usize = 16 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct BenchmarkSubmissionFailure {
    schema_version: String,
    run_id: String,
    submitted_at_utc: String,
    error: String,
}

/// Submit prepared benchmark material once, persisting intent before the scheduler effect.
///
/// A persisted matching handle is returned without calling `sbatch` again. An intent without a
/// handle is deliberately not retried because the prior scheduler effect may have succeeded before
/// its handle could be written; the attempt must be reconciled instead of duplicated.
///
/// # Errors
/// Returns an error when prepared material drifted, evidence conflicts, a prior intent is
/// unresolved, scheduler submission fails, or the returned handle cannot be persisted.
pub fn submit_prepared_benchmark_job(
    paths: &SlurmClientPaths,
    prepared: &PreparedBenchmarkSlurmJob,
) -> Result<SlurmJobHandle> {
    let scheduler_root = scheduler_root(prepared)?;
    recheck_material(prepared, &scheduler_root)?;
    let intent_path = scheduler_root.join("submission-intent.json");
    let intent = submission_intent(prepared)?;
    let handle_path = scheduler_root.join("job-handle.json");
    if regular_file_exists(&handle_path, "benchmark Slurm handle")? {
        let existing_intent: BenchmarkSlurmSubmissionIntent = read_json(&intent_path)?;
        validate_benchmark_slurm_submission_intent(&existing_intent)?;
        if existing_intent != intent {
            bail!("persisted benchmark submission intent conflicts with its Slurm handle");
        }
        return load_matching_handle(&handle_path, prepared);
    }

    if !create_intent(&intent_path, &intent)? {
        bail!(
            "benchmark submission intent has no persisted handle; reconcile scheduler state before retry"
        );
    }
    let handle = match submit_slurm_job(paths, &prepared.request) {
        Ok(handle) => handle,
        Err(error) => {
            let failure = BenchmarkSubmissionFailure {
                schema_version: SUBMISSION_FAILURE_SCHEMA_V1.to_string(),
                run_id: prepared.request.run_id.clone(),
                submitted_at_utc: prepared.request.submitted_at_utc.clone(),
                error: format!("{error:#}"),
            };
            ensure_json_identity(&scheduler_root.join("submission-failure.json"), &failure)?;
            return Err(error).context("submit prepared benchmark job");
        }
    };
    ensure_json_identity(&handle_path, &handle)?;
    Ok(handle)
}

/// Load and validate the scheduler handle retained for one benchmark execution specification.
///
/// This read-only recovery path remains valid after benchmark outputs exist. It verifies the
/// persisted specification, scheduler script, submission intent, and handle as one identity before
/// returning scheduler evidence to an observer.
///
/// # Errors
/// Returns an error when the specification is invalid, material is missing or changed, or retained
/// submission evidence conflicts with the execution identity.
pub fn load_benchmark_slurm_job_handle(spec: &BenchmarkExecutionSpec) -> Result<SlurmJobHandle> {
    validate_execution_spec(spec).context("validate benchmark execution specification")?;
    let result_root = Path::new(&spec.outputs.result_root);
    let canonical_result_root =
        result_root.canonicalize().context("canonicalize benchmark result root")?;
    if canonical_result_root != result_root {
        bail!("benchmark result root is not canonical");
    }
    let scheduler_root = canonical_result_root.join("scheduler");
    let scheduler_metadata = fs::symlink_metadata(&scheduler_root)
        .context("inspect persisted benchmark scheduler root")?;
    if scheduler_metadata.file_type().is_symlink() || !scheduler_metadata.is_dir() {
        bail!("persisted benchmark scheduler root is not a regular directory");
    }

    let persisted_spec: BenchmarkExecutionSpec =
        read_json(&canonical_result_root.join("execution-spec.json"))?;
    if persisted_spec != *spec {
        bail!("persisted benchmark execution specification conflicts with requested identity");
    }
    let script_path = scheduler_root.join("job.sbatch");
    if !regular_file_exists(&script_path, "benchmark Slurm script")? {
        bail!("persisted benchmark Slurm script is missing");
    }
    let intent_path = scheduler_root.join("submission-intent.json");
    let intent: BenchmarkSlurmSubmissionIntent = read_json(&intent_path)?;
    validate_benchmark_slurm_submission_intent(&intent)?;
    let material = intent.material();
    let prepared = PreparedBenchmarkSlurmJob {
        request: super::SlurmSubmitRequest {
            run_id: format!(
                "{}:{}:{}",
                spec.identity.run.run_id, spec.identity.tool_id, spec.identity.attempt_id
            ),
            dependencies: material.dependencies,
            dependency_condition: runner_dependency_condition(material.dependency_condition),
            array: material.array.as_ref().map(runner_array_spec),
            script_path,
            working_directory: PathBuf::from(&spec.source.repository_path),
            stdout_path: scheduler_root.join("stdout.log"),
            stderr_path: scheduler_root.join("stderr.log"),
            submitted_at_utc: material.submitted_at_utc,
        },
        execution_spec_sha256: bijux_dna_infra::hash_file_sha256(
            &canonical_result_root.join("execution-spec.json"),
        )
        .context("hash persisted benchmark execution specification")?,
        execution_unit_manifest_sha256: bijux_dna_infra::hash_file_sha256(
            &canonical_result_root.join("stage-plan-manifest.json"),
        )
        .context("hash persisted benchmark stage-plan manifest")?,
        script_sha256: bijux_dna_infra::hash_file_sha256(&scheduler_root.join("job.sbatch"))
            .context("hash persisted benchmark Slurm script")?,
    };
    if submission_intent(&prepared)? != intent {
        bail!("persisted benchmark submission intent conflicts with execution material");
    }
    recheck_material(&prepared, &scheduler_root)?;
    load_matching_handle(&scheduler_root.join("job-handle.json"), &prepared)
}

fn scheduler_root(prepared: &PreparedBenchmarkSlurmJob) -> Result<PathBuf> {
    let script_path = prepared
        .request
        .script_path
        .canonicalize()
        .context("canonicalize prepared benchmark script")?;
    if script_path.file_name().and_then(|name| name.to_str()) != Some("job.sbatch") {
        bail!("prepared benchmark script has an unsupported name");
    }
    let root = script_path.parent().context("prepared benchmark script has no parent")?;
    let metadata = fs::symlink_metadata(root).context("inspect benchmark scheduler root")?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        bail!("benchmark scheduler root is not a regular directory");
    }
    Ok(root.to_path_buf())
}

fn recheck_material(prepared: &PreparedBenchmarkSlurmJob, scheduler_root: &Path) -> Result<()> {
    let script_sha256 = bijux_dna_infra::hash_file_sha256(&prepared.request.script_path)
        .context("rehash prepared benchmark script")?;
    let execution_spec_path = scheduler_root
        .parent()
        .context("benchmark scheduler root has no attempt parent")?
        .join("execution-spec.json");
    let execution_spec_sha256 = bijux_dna_infra::hash_file_sha256(&execution_spec_path)
        .context("rehash prepared benchmark execution specification")?;
    let execution_unit_manifest_sha256 = bijux_dna_infra::hash_file_sha256(
        &execution_spec_path
            .parent()
            .context("benchmark execution specification has no parent")?
            .join("stage-plan-manifest.json"),
    )
    .context("rehash prepared benchmark stage-plan manifest")?;
    if script_sha256 != prepared.script_sha256
        || execution_spec_sha256 != prepared.execution_spec_sha256
        || execution_unit_manifest_sha256 != prepared.execution_unit_manifest_sha256
    {
        bail!("prepared benchmark scheduler material changed before submission");
    }
    Ok(())
}

fn submission_intent(
    prepared: &PreparedBenchmarkSlurmJob,
) -> Result<BenchmarkSlurmSubmissionIntent> {
    let intent = BenchmarkSlurmSubmissionIntent::from_material(BenchmarkSlurmSubmissionMaterial {
        schema_version: BENCHMARK_SLURM_SUBMISSION_INTENT_SCHEMA_V1.to_string(),
        run_id: prepared.request.run_id.clone(),
        dependencies: prepared.request.dependencies.clone(),
        dependency_condition: model_dependency_condition(prepared.request.dependency_condition),
        array: prepared.request.array.as_ref().map(model_array_spec),
        execution_spec_sha256: prepared.execution_spec_sha256.clone(),
        execution_unit_manifest_sha256: prepared.execution_unit_manifest_sha256.clone(),
        script_sha256: prepared.script_sha256.clone(),
        script_path: prepared.request.script_path.clone(),
        working_directory: prepared.request.working_directory.clone(),
        stdout_path: prepared.request.stdout_path.clone(),
        stderr_path: prepared.request.stderr_path.clone(),
        submitted_at_utc: prepared.request.submitted_at_utc.clone(),
    });
    validate_benchmark_slurm_submission_intent(&intent)?;
    Ok(intent)
}

fn model_dependency_condition(
    condition: SlurmDependencyCondition,
) -> BenchmarkSlurmDependencyCondition {
    match condition {
        SlurmDependencyCondition::AfterOk => BenchmarkSlurmDependencyCondition::AfterOk,
        SlurmDependencyCondition::AfterAny => BenchmarkSlurmDependencyCondition::AfterAny,
    }
}

fn runner_dependency_condition(
    condition: BenchmarkSlurmDependencyCondition,
) -> SlurmDependencyCondition {
    match condition {
        BenchmarkSlurmDependencyCondition::AfterOk => SlurmDependencyCondition::AfterOk,
        BenchmarkSlurmDependencyCondition::AfterAny => SlurmDependencyCondition::AfterAny,
    }
}

fn model_array_spec(array: &SlurmArraySpec) -> BenchmarkSlurmArraySpec {
    BenchmarkSlurmArraySpec {
        first_task_id: array.first_task_id,
        last_task_id: array.last_task_id,
        maximum_parallel_tasks: array.maximum_parallel_tasks,
    }
}

fn runner_array_spec(array: &BenchmarkSlurmArraySpec) -> SlurmArraySpec {
    SlurmArraySpec {
        first_task_id: array.first_task_id,
        last_task_id: array.last_task_id,
        maximum_parallel_tasks: array.maximum_parallel_tasks,
    }
}

fn create_intent(path: &Path, intent: &BenchmarkSlurmSubmissionIntent) -> Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                bail!("benchmark submission intent is not a regular file");
            }
            let existing: BenchmarkSlurmSubmissionIntent = read_json(path)?;
            validate_benchmark_slurm_submission_intent(&existing)?;
            if existing != *intent {
                bail!("benchmark submission intent conflicts with prepared material");
            }
            Ok(false)
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {
            bijux_dna_infra::atomic_create_json(path, intent)
                .context("persist benchmark submission intent")?;
            Ok(true)
        }
        Err(error) => Err(error).context("inspect benchmark submission intent"),
    }
}

fn load_matching_handle(
    path: &Path,
    prepared: &PreparedBenchmarkSlurmJob,
) -> Result<SlurmJobHandle> {
    let handle: SlurmJobHandle = read_json(path)?;
    if handle.schema_version != SLURM_JOB_HANDLE_SCHEMA_V1
        || handle.run_id != prepared.request.run_id
        || handle.dependencies != prepared.request.dependencies
        || handle.dependency_condition != prepared.request.dependency_condition
        || handle.array != prepared.request.array
        || handle.script_path != prepared.request.script_path
        || handle.working_directory != prepared.request.working_directory
        || handle.stdout_path != prepared.request.stdout_path
        || handle.stderr_path != prepared.request.stderr_path
        || handle.submitted_at_utc != prepared.request.submitted_at_utc
        || handle.job_id.is_empty()
        || !handle.job_id.bytes().all(|byte| byte.is_ascii_digit())
    {
        bail!("persisted benchmark Slurm handle conflicts with prepared material");
    }
    Ok(handle)
}

fn ensure_json_identity<T>(path: &Path, value: &T) -> Result<()>
where
    T: Serialize + serde::de::DeserializeOwned + PartialEq,
{
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                bail!("benchmark submission evidence is not a regular file");
            }
            let existing: T = read_json(path)?;
            if existing != *value {
                bail!("benchmark submission evidence conflicts with immutable content");
            }
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {
            bijux_dna_infra::atomic_create_json(path, value)
                .context("persist benchmark submission evidence")?;
        }
        Err(error) => return Err(error).context("inspect benchmark submission evidence"),
    }
    Ok(())
}

fn regular_file_exists(path: &Path, label: &str) -> Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_file() && !metadata.file_type().is_symlink() => Ok(true),
        Ok(_) => bail!("{label} is not a regular file"),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error).with_context(|| format!("inspect {label} {}", path.display())),
    }
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    if !regular_file_exists(path, "benchmark submission evidence")? {
        bail!("benchmark submission evidence is missing");
    }
    let bytes = bijux_dna_infra::read_to_end_bounded(path, MAX_EVIDENCE_BYTES)
        .with_context(|| format!("read benchmark submission evidence {}", path.display()))?;
    serde_json::from_slice(&bytes)
        .with_context(|| format!("parse benchmark submission evidence {}", path.display()))
}
