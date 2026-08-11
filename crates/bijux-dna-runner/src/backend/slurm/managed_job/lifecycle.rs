//! Duplicate-safe submission and one-shot scheduler evidence collection.

use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use anyhow::{bail, Context, Result};

use super::super::{
    collect_slurm_job, observe_slurm_job, submit_slurm_job, SlurmClientPaths, SlurmJobHandle,
    SlurmJobObservation, SlurmSubmitRequest, SLURM_JOB_HANDLE_SCHEMA_V1,
};
use super::contracts::{
    ManagedSlurmJobCollection, ManagedSlurmJobEvidence, ManagedSlurmJobSpec,
    PreparedManagedSlurmJob, SubmissionFailure, SubmissionIntent,
    MANAGED_SLURM_JOB_EVIDENCE_SCHEMA_V1, PREPARED_MANAGED_SLURM_JOB_SCHEMA_V1,
    SUBMISSION_FAILURE_SCHEMA_V1, SUBMISSION_INTENT_SCHEMA_V1,
};
use super::preparation::ensure_json_identity;
use super::validation::{canonical_directory, read_json, validate_spec, validate_timestamp};

/// Load and revalidate immutable prepared scheduler material.
///
/// # Errors
/// Returns an error when retained material is missing, malformed, or changed.
pub fn load_prepared_managed_slurm_job(
    spec: &ManagedSlurmJobSpec,
) -> Result<PreparedManagedSlurmJob> {
    validate_spec(spec)?;
    let scheduler_root = canonical_directory(&spec.attempt_root.join("scheduler"), "scheduler")?;
    let prepared: PreparedManagedSlurmJob = read_json(&scheduler_root.join("prepared-job.json"))?;
    recheck_prepared(spec, &prepared, &scheduler_root)?;
    Ok(prepared)
}

/// Submit one managed job after persisting intent and return immediately.
///
/// # Errors
/// Returns an error for identity drift, unresolved prior intent, or scheduler rejection.
pub fn submit_prepared_managed_slurm_job(
    paths: &SlurmClientPaths,
    spec: &ManagedSlurmJobSpec,
) -> Result<SlurmJobHandle> {
    let prepared = load_prepared_managed_slurm_job(spec)?;
    let scheduler_root = spec.attempt_root.join("scheduler");
    let intent = SubmissionIntent {
        schema_version: SUBMISSION_INTENT_SCHEMA_V1.to_string(),
        request: prepared.request.clone(),
        specification_sha256: prepared.specification_sha256.clone(),
        worker_sha256: prepared.worker_sha256.clone(),
        script_sha256: prepared.script_sha256.clone(),
    };
    let intent_path = scheduler_root.join("submission-intent.json");
    let handle_path = scheduler_root.join("job-handle.json");
    if regular_file_exists(&handle_path, "managed Slurm handle")? {
        if read_json::<SubmissionIntent>(&intent_path)? != intent {
            bail!("managed Slurm submission intent conflicts with persisted handle");
        }
        return load_matching_handle(&handle_path, &prepared.request);
    }
    if !create_intent(&intent_path, &intent)? {
        bail!("managed Slurm submission intent has no handle; reconcile before retry");
    }
    let handle = match submit_slurm_job(paths, &prepared.request) {
        Ok(handle) => handle,
        Err(error) => {
            ensure_json_identity(
                &scheduler_root.join("submission-failure.json"),
                &SubmissionFailure {
                    schema_version: SUBMISSION_FAILURE_SCHEMA_V1.to_string(),
                    run_id: prepared.request.run_id,
                    submitted_at_utc: prepared.request.submitted_at_utc,
                    error: format!("{error:#}"),
                },
            )?;
            return Err(error).context("submit managed Slurm job");
        }
    };
    ensure_json_identity(&handle_path, &handle)?;
    Ok(handle)
}

/// Load the persisted handle for one managed worker.
///
/// # Errors
/// Returns an error when scheduler identity evidence is missing or conflicts.
pub fn load_managed_slurm_job_handle(spec: &ManagedSlurmJobSpec) -> Result<SlurmJobHandle> {
    let prepared = load_prepared_managed_slurm_job(spec)?;
    load_matching_handle(&spec.attempt_root.join("scheduler/job-handle.json"), &prepared.request)
}

/// Observe one managed job once and return immediately.
///
/// # Errors
/// Returns an error for invalid evidence, timestamp, or scheduler response.
pub fn observe_managed_slurm_job(
    paths: &SlurmClientPaths,
    spec: &ManagedSlurmJobSpec,
    observed_at_utc: &str,
) -> Result<SlurmJobObservation> {
    validate_timestamp(observed_at_utc)?;
    observe_slurm_job(paths, &load_managed_slurm_job_handle(spec)?, observed_at_utc)
}

/// Collect one terminal managed job once; active jobs return without persisted collection.
///
/// # Errors
/// Returns an error for invalid scheduler evidence, missing logs, or conflicting collection data.
pub fn collect_managed_slurm_job(
    paths: &SlurmClientPaths,
    spec: &ManagedSlurmJobSpec,
    collected_at_utc: &str,
) -> Result<ManagedSlurmJobCollection> {
    validate_timestamp(collected_at_utc)?;
    let prepared = load_prepared_managed_slurm_job(spec)?;
    let handle = load_matching_handle(
        &spec.attempt_root.join("scheduler/job-handle.json"),
        &prepared.request,
    )?;
    let observation = collect_slurm_job(paths, &handle, collected_at_utc)?;
    if !observation.state.is_terminal() {
        return Ok(ManagedSlurmJobCollection { observation, evidence: None });
    }
    let evidence = ManagedSlurmJobEvidence {
        schema_version: MANAGED_SLURM_JOB_EVIDENCE_SCHEMA_V1.to_string(),
        run_id: spec.run_id.clone(),
        attempt_id: spec.attempt_id.clone(),
        repository_commit: spec.repository_commit.clone(),
        job_id: handle.job_id,
        specification_sha256: prepared.specification_sha256,
        worker_sha256: prepared.worker_sha256,
        script_sha256: prepared.script_sha256,
        accounting: observation
            .accounting
            .clone()
            .context("terminal managed Slurm observation lacks accounting")?,
        stdout_sha256: require_file_digest(&handle.stdout_path, "stdout")?,
        stderr_sha256: require_file_digest(&handle.stderr_path, "stderr")?,
        collected_at_utc: collected_at_utc.to_string(),
    };
    let collection_path = spec.attempt_root.join("scheduler/collection.json");
    if regular_file_exists(&collection_path, "managed Slurm collection")? {
        let retained: ManagedSlurmJobEvidence = read_json(&collection_path)?;
        let mut expected = evidence.clone();
        expected.collected_at_utc.clone_from(&retained.collected_at_utc);
        if retained != expected {
            bail!("persisted managed Slurm collection conflicts with terminal evidence");
        }
        return Ok(ManagedSlurmJobCollection { observation, evidence: Some(retained) });
    }
    ensure_json_identity(&collection_path, &evidence)?;
    Ok(ManagedSlurmJobCollection { observation, evidence: Some(evidence) })
}

fn recheck_prepared(
    spec: &ManagedSlurmJobSpec,
    prepared: &PreparedManagedSlurmJob,
    scheduler_root: &Path,
) -> Result<()> {
    if prepared.schema_version != PREPARED_MANAGED_SLURM_JOB_SCHEMA_V1
        || prepared.request.script_path != scheduler_root.join("job.sbatch")
        || prepared.worker_sha256 != spec.worker_sha256
        || prepared.request.array.is_some()
    {
        bail!("prepared managed Slurm material conflicts with specification");
    }
    if bijux_dna_infra::hash_file_sha256(&prepared.request.script_path)? != prepared.script_sha256
        || bijux_dna_infra::hash_file_sha256(
            &spec.attempt_root.join("managed-slurm-job-spec.json"),
        )? != prepared.specification_sha256
    {
        bail!("prepared managed Slurm material changed after preparation");
    }
    Ok(())
}

fn create_intent(path: &Path, intent: &SubmissionIntent) -> Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                bail!("managed Slurm submission intent is not a regular file");
            }
            if read_json::<SubmissionIntent>(path)? != *intent {
                bail!("managed Slurm submission intent conflicts with prepared material");
            }
            Ok(false)
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {
            bijux_dna_infra::atomic_create_json(path, intent)?;
            Ok(true)
        }
        Err(error) => Err(error).context("inspect managed Slurm submission intent"),
    }
}

fn load_matching_handle(path: &Path, request: &SlurmSubmitRequest) -> Result<SlurmJobHandle> {
    let handle: SlurmJobHandle = read_json(path)?;
    if handle.schema_version != SLURM_JOB_HANDLE_SCHEMA_V1
        || handle.run_id != request.run_id
        || handle.dependencies != request.dependencies
        || handle.dependency_condition != request.dependency_condition
        || handle.array != request.array
        || handle.script_path != request.script_path
        || handle.working_directory != request.working_directory
        || handle.stdout_path != request.stdout_path
        || handle.stderr_path != request.stderr_path
        || handle.submitted_at_utc != request.submitted_at_utc
        || handle.job_id.is_empty()
        || !handle.job_id.bytes().all(|byte| byte.is_ascii_digit())
    {
        bail!("persisted managed Slurm handle conflicts with prepared material");
    }
    Ok(handle)
}

fn regular_file_exists(path: &Path, label: &str) -> Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => {
            bail!("{label} is not a regular file")
        }
        Ok(_) => Ok(true),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error).with_context(|| format!("inspect {label}")),
    }
}

fn require_file_digest(path: &Path, label: &str) -> Result<String> {
    if !regular_file_exists(path, label)? {
        bail!("terminal managed Slurm {label} log is missing");
    }
    Ok(bijux_dna_infra::hash_file_sha256(path)?)
}
