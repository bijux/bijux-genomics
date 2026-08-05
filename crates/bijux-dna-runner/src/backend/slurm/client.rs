//! Owner: bijux-dna-runner
//! Bounded non-waiting Slurm command execution.

use std::path::Path;
use std::process::Command;

use anyhow::{bail, Context, Result};

use super::parsing::{parse_accounting, parse_job_id, parse_queue_state};
use super::{
    SlurmBatchSubmission, SlurmClientPaths, SlurmEvidenceSource, SlurmJobHandle,
    SlurmJobObservation, SlurmJobState, SlurmSubmissionFailure, SlurmSubmitRequest,
    SLURM_JOB_HANDLE_SCHEMA_V1,
};

/// Submit one batch script and return immediately after `sbatch` returns its job identity.
///
/// # Errors
/// Returns an error when request paths are invalid, `sbatch` fails, or its output is malformed.
pub fn submit_slurm_job(
    paths: &SlurmClientPaths,
    request: &SlurmSubmitRequest,
) -> Result<SlurmJobHandle> {
    validate_submit_request(request)?;
    let output = execute(
        &paths.sbatch,
        &[
            "--parsable".to_string(),
            "--chdir".to_string(),
            request.working_directory.display().to_string(),
            "--output".to_string(),
            request.stdout_path.display().to_string(),
            "--error".to_string(),
            request.stderr_path.display().to_string(),
            request.script_path.display().to_string(),
        ],
    )?;
    Ok(SlurmJobHandle {
        schema_version: SLURM_JOB_HANDLE_SCHEMA_V1.to_string(),
        run_id: request.run_id.clone(),
        job_id: parse_job_id(&output)?,
        script_path: request.script_path.clone(),
        working_directory: request.working_directory.clone(),
        stdout_path: request.stdout_path.clone(),
        stderr_path: request.stderr_path.clone(),
        submitted_at_utc: request.submitted_at_utc.clone(),
    })
}

/// Submit a bounded batch without polling or waiting for any job.
///
/// # Errors
/// Returns an error when the request count exceeds the explicit bound. Individual submission
/// failures are retained beside every successful handle in the returned report.
pub fn submit_slurm_batch(
    paths: &SlurmClientPaths,
    requests: &[SlurmSubmitRequest],
    maximum_submissions: usize,
) -> Result<SlurmBatchSubmission> {
    if maximum_submissions == 0 || requests.len() > maximum_submissions {
        bail!("Slurm submission batch exceeds its explicit bound");
    }
    let mut submitted = Vec::new();
    let mut failures = Vec::new();
    for request in requests {
        match submit_slurm_job(paths, request) {
            Ok(handle) => submitted.push(handle),
            Err(error) => failures.push(SlurmSubmissionFailure {
                run_id: request.run_id.clone(),
                error: format!("{error:#}"),
            }),
        }
    }
    Ok(SlurmBatchSubmission { submitted, failures })
}

/// Observe one job once, consulting accounting only after it leaves the queue.
///
/// # Errors
/// Returns an error when scheduler commands fail or return malformed evidence.
pub fn observe_slurm_job(
    paths: &SlurmClientPaths,
    handle: &SlurmJobHandle,
    observed_at_utc: &str,
) -> Result<SlurmJobObservation> {
    validate_handle(handle)?;
    require_text(observed_at_utc, "observation timestamp")?;
    let queue = execute_queue(
        &paths.squeue,
        &[
            "--noheader".to_string(),
            "--jobs".to_string(),
            handle.job_id.clone(),
            "--format".to_string(),
            "%T".to_string(),
        ],
    )?;
    if let Some(state) = parse_queue_state(&queue)? {
        return Ok(SlurmJobObservation {
            job_id: handle.job_id.clone(),
            state,
            source: SlurmEvidenceSource::Queue,
            observed_at_utc: observed_at_utc.to_string(),
            accounting: None,
        });
    }
    let accounting_output = execute(
        &paths.sacct,
        &[
            "--noheader".to_string(),
            "--parsable2".to_string(),
            "--jobs".to_string(),
            handle.job_id.clone(),
            "--format".to_string(),
            "JobIDRaw,State,ExitCode,ElapsedRaw,TotalCPU,MaxRSS,NodeList,Reason".to_string(),
        ],
    )?;
    let accounting = parse_accounting(&accounting_output, &handle.job_id)?;
    let state = accounting.as_ref().map_or(SlurmJobState::AccountingPending, |row| row.state);
    Ok(SlurmJobObservation {
        job_id: handle.job_id.clone(),
        state,
        source: SlurmEvidenceSource::Accounting,
        observed_at_utc: observed_at_utc.to_string(),
        accounting,
    })
}

/// Collect terminal accounting once without mutating scheduler or evidence state.
///
/// Repeating this operation with unchanged scheduler evidence is idempotent.
///
/// # Errors
/// Returns an error when scheduler commands fail or return malformed evidence.
pub fn collect_slurm_job(
    paths: &SlurmClientPaths,
    handle: &SlurmJobHandle,
    observed_at_utc: &str,
) -> Result<SlurmJobObservation> {
    observe_slurm_job(paths, handle, observed_at_utc)
}

/// Request cancellation once and return after `scancel` exits.
///
/// # Errors
/// Returns an error when the handle is invalid or `scancel` fails.
pub fn cancel_slurm_job(paths: &SlurmClientPaths, handle: &SlurmJobHandle) -> Result<()> {
    validate_handle(handle)?;
    execute(&paths.scancel, std::slice::from_ref(&handle.job_id))?;
    Ok(())
}

fn execute_queue(executable: &Path, arguments: &[String]) -> Result<String> {
    let output = Command::new(executable)
        .args(arguments)
        .output()
        .with_context(|| format!("execute Slurm command {}", executable.display()))?;
    let stderr = String::from_utf8_lossy(&output.stderr);
    let absent_from_queue = output.status.code() == Some(1)
        && output.stdout.is_empty()
        && stderr.trim() == "slurm_load_jobs error: Invalid job id specified";
    if absent_from_queue {
        return Ok(String::new());
    }
    if !output.status.success() {
        bail!(
            "Slurm command {} failed with {}: {}",
            executable.display(),
            output.status,
            stderr.trim()
        );
    }
    String::from_utf8(output.stdout).context("Slurm command stdout is not UTF-8")
}

fn execute(executable: &Path, arguments: &[String]) -> Result<String> {
    let output = Command::new(executable)
        .args(arguments)
        .output()
        .with_context(|| format!("execute Slurm command {}", executable.display()))?;
    if !output.status.success() {
        bail!(
            "Slurm command {} failed with {}: {}",
            executable.display(),
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    String::from_utf8(output.stdout).context("Slurm command stdout is not UTF-8")
}

fn validate_submit_request(request: &SlurmSubmitRequest) -> Result<()> {
    require_text(&request.run_id, "run id")?;
    require_text(&request.submitted_at_utc, "submission timestamp")?;
    if !request.script_path.is_file() {
        bail!("Slurm submission script is missing: {}", request.script_path.display());
    }
    if !request.working_directory.is_dir() {
        bail!("Slurm working directory is missing: {}", request.working_directory.display());
    }
    for (path, label) in [(&request.stdout_path, "stdout"), (&request.stderr_path, "stderr")] {
        if !path.is_absolute() {
            bail!("Slurm {label} path must be absolute");
        }
    }
    Ok(())
}

fn validate_handle(handle: &SlurmJobHandle) -> Result<()> {
    if handle.schema_version != SLURM_JOB_HANDLE_SCHEMA_V1
        || handle.job_id.is_empty()
        || !handle.job_id.bytes().all(|byte| byte.is_ascii_digit())
    {
        bail!("Slurm job handle identity is invalid");
    }
    require_text(&handle.run_id, "run id")?;
    require_text(&handle.submitted_at_utc, "submission timestamp")
}

fn require_text(value: &str, label: &str) -> Result<()> {
    if value.trim().is_empty() || value != value.trim() {
        bail!("Slurm {label} is empty or malformed");
    }
    Ok(())
}
