//! Owner: bijux-dna-runner
//! Restart-safe non-waiting lifecycle for container qualification jobs.

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

use super::contracts::{
    ContainerQualificationCollection, ContainerQualificationSchedulerEvidence,
    ContainerQualificationSchedulerStatus, ContainerQualificationSlurmSpec,
    PreparedContainerQualificationSlurmJob, CONTAINER_QUALIFICATION_EVIDENCE_SCHEMA_V1,
    CONTAINER_QUALIFICATION_PREPARED_JOB_SCHEMA_V1,
};
use super::validation::{
    canonical_directory, ensure_json_identity, read_json, require_utc_timestamp,
    validate_specification,
};
use crate::backend::slurm::{
    cancel_slurm_job, collect_slurm_job, observe_slurm_job, submit_slurm_job, SlurmArraySpec,
    SlurmClientPaths, SlurmDependencyCondition, SlurmJobHandle, SlurmJobObservation, SlurmJobState,
    SlurmSubmitRequest, SLURM_JOB_HANDLE_SCHEMA_V1,
};

const SUBMISSION_INTENT_SCHEMA_V1: &str =
    "bijux.runner.container_qualification_submission_intent.v1";
const SUBMISSION_FAILURE_SCHEMA_V1: &str =
    "bijux.runner.container_qualification_submission_failure.v1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct SubmissionIntent {
    schema_version: String,
    run_id: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    dependencies: Vec<String>,
    #[serde(default, skip_serializing_if = "SlurmDependencyCondition::is_after_ok")]
    dependency_condition: SlurmDependencyCondition,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    array: Option<SlurmArraySpec>,
    specification_sha256: String,
    script_sha256: String,
    script_path: PathBuf,
    working_directory: PathBuf,
    stdout_path: PathBuf,
    stderr_path: PathBuf,
    submitted_at_utc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct SubmissionFailure {
    schema_version: String,
    run_id: String,
    submitted_at_utc: String,
    error: String,
}

/// Submit one prepared qualification job after persisting immutable scheduler intent.
///
/// # Errors
/// Returns an error for changed material, unresolved intent, conflicting evidence, scheduler
/// rejection, or failure to persist the returned handle.
pub fn submit_prepared_container_qualification_job(
    paths: &SlurmClientPaths,
    prepared: &PreparedContainerQualificationSlurmJob,
) -> Result<SlurmJobHandle> {
    let scheduler_root = scheduler_root(&prepared.request)?;
    recheck_prepared(prepared, &scheduler_root)?;
    let persisted: PreparedContainerQualificationSlurmJob =
        read_json(&scheduler_root.join("prepared-job.json"))?;
    if persisted != *prepared {
        bail!("prepared qualification job conflicts with immutable scheduler evidence");
    }
    let intent = submission_intent(prepared);
    let intent_path = scheduler_root.join("submission-intent.json");
    let handle_path = scheduler_root.join("job-handle.json");
    if regular_file_exists(&handle_path, "qualification Slurm handle")? {
        let persisted_intent: SubmissionIntent = read_json(&intent_path)?;
        if persisted_intent != intent {
            bail!("qualification submission intent conflicts with persisted handle");
        }
        return load_matching_handle(&handle_path, &prepared.request);
    }
    if !create_intent(&intent_path, &intent)? {
        bail!(
            "qualification submission intent has no handle; reconcile scheduler state before retry"
        );
    }
    let handle = match submit_slurm_job(paths, &prepared.request) {
        Ok(handle) => handle,
        Err(error) => {
            ensure_json_identity(
                &scheduler_root.join("submission-failure.json"),
                &SubmissionFailure {
                    schema_version: SUBMISSION_FAILURE_SCHEMA_V1.to_string(),
                    run_id: prepared.request.run_id.clone(),
                    submitted_at_utc: prepared.request.submitted_at_utc.clone(),
                    error: format!("{error:#}"),
                },
            )?;
            return Err(error).context("submit prepared container qualification job");
        }
    };
    ensure_json_identity(&handle_path, &handle)?;
    Ok(handle)
}

/// Load immutable prepared qualification material for disconnect-safe resume.
///
/// # Errors
/// Returns an error for missing, changed, or conflicting specification and scheduler evidence.
pub fn load_prepared_container_qualification_slurm_job(
    spec: &ContainerQualificationSlurmSpec,
) -> Result<PreparedContainerQualificationSlurmJob> {
    validate_specification(spec)?;
    let evidence_root = canonical_directory(&spec.evidence_root, "qualification evidence root")?;
    let persisted_spec: ContainerQualificationSlurmSpec =
        read_json(&evidence_root.join("qualification-spec.json"))?;
    if persisted_spec != *spec {
        bail!("persisted qualification specification conflicts with requested identity");
    }
    let scheduler_root =
        canonical_directory(&evidence_root.join("scheduler"), "qualification scheduler root")?;
    let prepared: PreparedContainerQualificationSlurmJob =
        read_json(&scheduler_root.join("prepared-job.json"))?;
    require_utc_timestamp(&prepared.request.submitted_at_utc, "submission timestamp")?;
    if prepared.schema_version != CONTAINER_QUALIFICATION_PREPARED_JOB_SCHEMA_V1
        || prepared.request.run_id != format!("{}:{}", spec.run_id, spec.tool_id)
        || prepared.request.dependencies != spec.dependencies
        || prepared.request.array.is_some()
        || prepared.request.script_path != scheduler_root.join("job.sbatch")
        || prepared.request.working_directory != spec.repository_path
        || prepared.request.stdout_path != scheduler_root.join("stdout.log")
        || prepared.request.stderr_path != scheduler_root.join("stderr.log")
    {
        bail!("prepared qualification job conflicts with requested specification");
    }
    recheck_prepared(&prepared, &scheduler_root)?;
    Ok(prepared)
}

/// Load one persisted qualification scheduler handle after revalidating its identity chain.
///
/// # Errors
/// Returns an error for missing, changed, or conflicting scheduler evidence.
pub fn load_container_qualification_slurm_job_handle(
    spec: &ContainerQualificationSlurmSpec,
) -> Result<SlurmJobHandle> {
    let prepared = load_prepared_container_qualification_slurm_job(spec)?;
    let scheduler_root = scheduler_root(&prepared.request)?;
    let intent: SubmissionIntent = read_json(&scheduler_root.join("submission-intent.json"))?;
    if submission_intent(&prepared) != intent {
        bail!("qualification submission intent conflicts with scheduler material");
    }
    load_matching_handle(&scheduler_root.join("job-handle.json"), &prepared.request)
}

/// Observe one qualification job once without polling or mutating scheduler evidence.
///
/// # Errors
/// Returns an error when retained identity or scheduler evidence is invalid.
pub fn observe_container_qualification_slurm_job(
    paths: &SlurmClientPaths,
    spec: &ContainerQualificationSlurmSpec,
    observed_at_utc: &str,
) -> Result<SlurmJobObservation> {
    require_utc_timestamp(observed_at_utc, "observation timestamp")?;
    let handle = load_container_qualification_slurm_job_handle(spec)?;
    observe_slurm_job(paths, &handle, observed_at_utc)
}

/// Collect once and publish immutable scheduler evidence only for a terminal qualification job.
///
/// Active and accounting-pending jobs return immediately. Retained terminal evidence is returned
/// idempotently without consulting the scheduler again.
///
/// # Errors
/// Returns an error for changed authority, malformed scheduler output, incomplete successful
/// execution evidence, or conflicting terminal publication.
pub fn collect_container_qualification_slurm_job(
    paths: &SlurmClientPaths,
    spec: &ContainerQualificationSlurmSpec,
    collected_at_utc: &str,
) -> Result<ContainerQualificationCollection> {
    require_utc_timestamp(collected_at_utc, "collection timestamp")?;
    validate_specification(spec)?;
    let handle = load_container_qualification_slurm_job_handle(spec)?;
    let terminal_path = spec.evidence_root.join("scheduler-evidence.json");
    if regular_file_exists(&terminal_path, "qualification terminal evidence")? {
        let evidence: ContainerQualificationSchedulerEvidence = read_json(&terminal_path)?;
        validate_retained_evidence(spec, &handle, &evidence)?;
        let observation = SlurmJobObservation {
            job_id: evidence.job_id.clone(),
            state: evidence.accounting.state,
            source: crate::backend::slurm::SlurmEvidenceSource::Accounting,
            observed_at_utc: evidence.collected_at_utc.clone(),
            accounting: Some(evidence.accounting.clone()),
        };
        return Ok(ContainerQualificationCollection { observation, evidence: Some(evidence) });
    }

    let observation = collect_slurm_job(paths, &handle, collected_at_utc)?;
    if !observation.state.is_terminal() {
        return Ok(ContainerQualificationCollection { observation, evidence: None });
    }
    let accounting = observation
        .accounting
        .clone()
        .context("terminal qualification observation lacks accounting")?;
    let prepared = prepared_from_evidence(spec, &handle)?;
    let scheduler_root = spec.evidence_root.join("scheduler");
    let stdout_sha256 = hash_required_log(&scheduler_root.join("stdout.log"), "stdout")?;
    let stderr_sha256 = hash_required_log(&scheduler_root.join("stderr.log"), "stderr")?;
    let execution_evidence = spec.result_root.join("evidence.json");
    let scheduler_succeeded = accounting.state == SlurmJobState::Completed
        && accounting.exit_code == 0
        && accounting.signal == 0;
    let (execution_evidence_sha256, failure) = if scheduler_succeeded {
        match validate_successful_execution_evidence(spec, &execution_evidence) {
            Ok(sha256) => (Some(sha256), None),
            Err(error) => (None, Some(format!("{error:#}"))),
        }
    } else {
        (None, Some(terminal_failure(&accounting)))
    };
    let passed = execution_evidence_sha256.is_some();
    let evidence = ContainerQualificationSchedulerEvidence {
        schema_version: CONTAINER_QUALIFICATION_EVIDENCE_SCHEMA_V1.to_string(),
        status: if passed {
            ContainerQualificationSchedulerStatus::Passed
        } else {
            ContainerQualificationSchedulerStatus::Failed
        },
        run_id: spec.run_id.clone(),
        tool_id: spec.tool_id.clone(),
        repository_commit: spec.repository_commit.clone(),
        job_id: handle.job_id,
        specification_sha256: prepared.specification_sha256,
        script_sha256: prepared.script_sha256,
        governance_sha256: spec.governance_sha256.clone(),
        pipeline_sha256: spec.pipeline_sha256.clone(),
        credential_reference_sha256: spec.credential_reference_sha256.clone(),
        executor_program_sha256: spec.executor_program_sha256.clone(),
        executor_script_sha256: spec.executor_script_sha256.clone(),
        accounting: accounting.clone(),
        stdout_sha256,
        stderr_sha256,
        execution_evidence_sha256,
        collected_at_utc: collected_at_utc.to_string(),
        failure,
    };
    ensure_json_identity(&terminal_path, &evidence)?;
    Ok(ContainerQualificationCollection { observation, evidence: Some(evidence) })
}

/// Request cancellation once and return as soon as `scancel` exits.
///
/// # Errors
/// Returns an error when retained identity is invalid or scheduler cancellation fails.
pub fn cancel_container_qualification_slurm_job(
    paths: &SlurmClientPaths,
    spec: &ContainerQualificationSlurmSpec,
) -> Result<()> {
    let handle = load_container_qualification_slurm_job_handle(spec)?;
    cancel_slurm_job(paths, &handle)
}

fn validate_successful_execution_evidence(
    spec: &ContainerQualificationSlurmSpec,
    path: &Path,
) -> Result<String> {
    if !regular_file_exists(path, "qualification execution evidence")? {
        bail!("successful qualification scheduler job lacks execution evidence");
    }
    let evidence: serde_json::Value = read_json(path)?;
    if evidence.get("run_id").and_then(serde_json::Value::as_str) != Some(spec.run_id.as_str())
        || evidence.get("status").and_then(serde_json::Value::as_str) != Some("passed")
        || evidence.get("evidence_kind").and_then(serde_json::Value::as_str)
            != Some("qualification")
        || evidence.get("repository_commit").and_then(serde_json::Value::as_str)
            != Some(spec.repository_commit.as_str())
        || evidence.get("repository_path").and_then(serde_json::Value::as_str)
            != spec.repository_path.to_str()
        || evidence.get("governance_sha256").and_then(serde_json::Value::as_str)
            != Some(spec.governance_sha256.as_str())
        || evidence.get("pipeline_sha256").and_then(serde_json::Value::as_str)
            != Some(spec.pipeline_sha256.as_str())
        || evidence.get("credential_reference_sha256").and_then(serde_json::Value::as_str)
            != Some(spec.credential_reference_sha256.as_str())
    {
        bail!("qualification execution evidence conflicts with scheduler specification");
    }
    Ok(bijux_dna_infra::hash_file_sha256(path)?)
}

fn validate_retained_evidence(
    spec: &ContainerQualificationSlurmSpec,
    handle: &SlurmJobHandle,
    evidence: &ContainerQualificationSchedulerEvidence,
) -> Result<()> {
    let prepared = prepared_from_evidence(spec, handle)?;
    if evidence.schema_version != CONTAINER_QUALIFICATION_EVIDENCE_SCHEMA_V1
        || evidence.run_id != spec.run_id
        || evidence.tool_id != spec.tool_id
        || evidence.repository_commit != spec.repository_commit
        || evidence.job_id != handle.job_id
        || evidence.specification_sha256 != prepared.specification_sha256
        || evidence.script_sha256 != prepared.script_sha256
        || evidence.governance_sha256 != spec.governance_sha256
        || evidence.pipeline_sha256 != spec.pipeline_sha256
        || evidence.credential_reference_sha256 != spec.credential_reference_sha256
        || evidence.executor_program_sha256 != spec.executor_program_sha256
        || evidence.executor_script_sha256 != spec.executor_script_sha256
        || evidence.accounting.job_id != handle.job_id
        || !evidence.accounting.state.is_terminal()
    {
        bail!("retained qualification scheduler evidence conflicts with requested identity");
    }
    require_utc_timestamp(&evidence.collected_at_utc, "retained collection timestamp")?;
    let scheduler_root = spec.evidence_root.join("scheduler");
    if evidence.stdout_sha256 != hash_required_log(&scheduler_root.join("stdout.log"), "stdout")?
        || evidence.stderr_sha256
            != hash_required_log(&scheduler_root.join("stderr.log"), "stderr")?
    {
        bail!("retained qualification scheduler evidence conflicts with scheduler logs");
    }
    match evidence.status {
        ContainerQualificationSchedulerStatus::Passed => {
            let observed_execution = validate_successful_execution_evidence(
                spec,
                &spec.result_root.join("evidence.json"),
            )?;
            if Some(observed_execution) != evidence.execution_evidence_sha256
                || evidence.failure.is_some()
                || evidence.accounting.state != SlurmJobState::Completed
                || evidence.accounting.exit_code != 0
                || evidence.accounting.signal != 0
            {
                bail!("retained passed qualification evidence is inconsistent");
            }
        }
        ContainerQualificationSchedulerStatus::Failed => {
            if evidence.execution_evidence_sha256.is_some()
                || evidence.failure.as_deref().is_none_or(str::is_empty)
            {
                bail!("retained failed qualification evidence is inconsistent");
            }
            if evidence.accounting.state == SlurmJobState::Completed
                && evidence.accounting.exit_code == 0
                && evidence.accounting.signal == 0
                && validate_successful_execution_evidence(
                    spec,
                    &spec.result_root.join("evidence.json"),
                )
                .is_ok()
            {
                bail!("retained failed qualification evidence now has passing execution evidence");
            }
        }
    }
    Ok(())
}

fn prepared_from_evidence(
    spec: &ContainerQualificationSlurmSpec,
    handle: &SlurmJobHandle,
) -> Result<PreparedContainerQualificationSlurmJob> {
    Ok(PreparedContainerQualificationSlurmJob {
        schema_version: CONTAINER_QUALIFICATION_PREPARED_JOB_SCHEMA_V1.to_string(),
        request: SlurmSubmitRequest {
            run_id: handle.run_id.clone(),
            dependencies: handle.dependencies.clone(),
            dependency_condition: handle.dependency_condition,
            array: handle.array.clone(),
            script_path: handle.script_path.clone(),
            working_directory: handle.working_directory.clone(),
            stdout_path: handle.stdout_path.clone(),
            stderr_path: handle.stderr_path.clone(),
            submitted_at_utc: handle.submitted_at_utc.clone(),
        },
        specification_sha256: bijux_dna_infra::hash_file_sha256(
            &spec.evidence_root.join("qualification-spec.json"),
        )?,
        script_sha256: bijux_dna_infra::hash_file_sha256(
            &spec.evidence_root.join("scheduler/job.sbatch"),
        )?,
    })
}

fn scheduler_root(request: &SlurmSubmitRequest) -> Result<PathBuf> {
    let path = request.script_path.canonicalize().context("canonicalize qualification script")?;
    if path.file_name().and_then(|name| name.to_str()) != Some("job.sbatch") {
        bail!("prepared qualification script has an unsupported name");
    }
    canonical_directory(
        path.parent().context("qualification script has no parent")?,
        "qualification scheduler root",
    )
}

fn recheck_prepared(
    prepared: &PreparedContainerQualificationSlurmJob,
    scheduler_root: &Path,
) -> Result<()> {
    let script_sha256 = bijux_dna_infra::hash_file_sha256(&prepared.request.script_path)?;
    let specification_sha256 = bijux_dna_infra::hash_file_sha256(
        &scheduler_root
            .parent()
            .context("qualification scheduler root has no evidence parent")?
            .join("qualification-spec.json"),
    )?;
    if script_sha256 != prepared.script_sha256
        || specification_sha256 != prepared.specification_sha256
    {
        bail!("prepared qualification scheduler material changed");
    }
    Ok(())
}

fn submission_intent(prepared: &PreparedContainerQualificationSlurmJob) -> SubmissionIntent {
    SubmissionIntent {
        schema_version: SUBMISSION_INTENT_SCHEMA_V1.to_string(),
        run_id: prepared.request.run_id.clone(),
        dependencies: prepared.request.dependencies.clone(),
        dependency_condition: prepared.request.dependency_condition,
        array: prepared.request.array.clone(),
        specification_sha256: prepared.specification_sha256.clone(),
        script_sha256: prepared.script_sha256.clone(),
        script_path: prepared.request.script_path.clone(),
        working_directory: prepared.request.working_directory.clone(),
        stdout_path: prepared.request.stdout_path.clone(),
        stderr_path: prepared.request.stderr_path.clone(),
        submitted_at_utc: prepared.request.submitted_at_utc.clone(),
    }
}

fn create_intent(path: &Path, intent: &SubmissionIntent) -> Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                bail!("qualification submission intent is not a regular file");
            }
            if read_json::<SubmissionIntent>(path)? != *intent {
                bail!("qualification submission intent conflicts with prepared material");
            }
            Ok(false)
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {
            bijux_dna_infra::atomic_create_json(path, intent)?;
            Ok(true)
        }
        Err(error) => Err(error).context("inspect qualification submission intent"),
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
        bail!("persisted qualification Slurm handle conflicts with prepared material");
    }
    Ok(handle)
}

fn regular_file_exists(path: &Path, label: &str) -> Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_file() && !metadata.file_type().is_symlink() => Ok(true),
        Ok(_) => bail!("{label} is not a regular file"),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error).with_context(|| format!("inspect {label}")),
    }
}

fn hash_required_log(path: &Path, label: &str) -> Result<String> {
    if !regular_file_exists(path, &format!("qualification {label} log"))? {
        bail!("qualification {label} log is missing");
    }
    bijux_dna_infra::hash_file_sha256(path).with_context(|| format!("hash {label} log"))
}

fn terminal_failure(accounting: &crate::backend::slurm::SlurmAccounting) -> String {
    format!(
        "Slurm terminal state {:?} with exit code {} and signal {}",
        accounting.state, accounting.exit_code, accounting.signal
    )
}
