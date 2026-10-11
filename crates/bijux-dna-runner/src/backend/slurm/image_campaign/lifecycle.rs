//! Owner: bijux-dna-runner
//! Restart-safe non-waiting Slurm lifecycle for container image campaigns.

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

use super::contracts::{
    ImageCampaignCollection, ImageCampaignEvidence, ImageCampaignEvidenceStatus,
    ImageCampaignSlurmSpec, PreparedImageCampaignSlurmJob, IMAGE_CAMPAIGN_EVIDENCE_SCHEMA_V1,
    IMAGE_CAMPAIGN_PREPARED_JOB_SCHEMA_V1,
};
use super::validation::{
    canonical_directory, ensure_json_identity, read_json, validate_specification,
    validate_submission_authorities,
};
use crate::backend::slurm::{
    collect_slurm_job, observe_slurm_job, submit_slurm_job, SlurmArraySpec, SlurmClientPaths,
    SlurmDependencyCondition, SlurmJobHandle, SlurmJobObservation, SlurmJobState,
    SLURM_JOB_HANDLE_SCHEMA_V1,
};

const SUBMISSION_INTENT_SCHEMA_V1: &str = "bijux.runner.image_campaign_submission_intent.v1";
const SUBMISSION_FAILURE_SCHEMA_V1: &str = "bijux.runner.image_campaign_submission_failure.v1";

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

/// Submit one prepared image campaign after persisting immutable intent.
///
/// # Errors
/// Returns an error for material drift, unresolved prior intent, scheduler rejection, or
/// conflicting evidence.
pub fn submit_prepared_image_campaign_job(
    paths: &SlurmClientPaths,
    prepared: &PreparedImageCampaignSlurmJob,
) -> Result<SlurmJobHandle> {
    let scheduler_root = scheduler_root(&prepared.request.script_path)?;
    recheck_prepared(prepared, &scheduler_root)?;
    let specification: ImageCampaignSlurmSpec = read_json(
        &scheduler_root
            .parent()
            .context("image campaign scheduler root lacks attempt root")?
            .join("image-campaign-spec.json"),
    )?;
    validate_specification(&specification)?;
    validate_submission_authorities(&specification)?;
    let persisted: PreparedImageCampaignSlurmJob =
        read_json(&scheduler_root.join("prepared-job.json"))?;
    if persisted != *prepared {
        bail!("prepared image campaign conflicts with immutable scheduler evidence");
    }
    let intent = submission_intent(prepared);
    let intent_path = scheduler_root.join("submission-intent.json");
    let handle_path = scheduler_root.join("job-handle.json");
    if regular_file_exists(&handle_path, "image campaign Slurm handle")? {
        if read_json::<SubmissionIntent>(&intent_path)? != intent {
            bail!("image campaign submission intent conflicts with persisted handle");
        }
        return load_matching_handle(&handle_path, &prepared.request);
    }
    if !create_intent(&intent_path, &intent)? {
        bail!("image campaign submission intent has no handle; reconcile scheduler state before retry");
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
            return Err(error).context("submit prepared image campaign");
        }
    };
    ensure_json_identity(&handle_path, &handle)?;
    Ok(handle)
}

/// Load prepared scheduler material after revalidating exact authorities.
///
/// # Errors
/// Returns an error for missing, changed, or conflicting material.
pub fn load_prepared_image_campaign_slurm_job(
    spec: &ImageCampaignSlurmSpec,
) -> Result<PreparedImageCampaignSlurmJob> {
    validate_specification(spec)?;
    let persisted_spec: ImageCampaignSlurmSpec =
        read_json(&spec.attempt_root.join("image-campaign-spec.json"))?;
    if persisted_spec != *spec {
        bail!("image campaign specification conflicts with immutable attempt evidence");
    }
    let scheduler_root =
        canonical_directory(&spec.attempt_root.join("scheduler"), "image campaign scheduler root")?;
    let prepared: PreparedImageCampaignSlurmJob =
        read_json(&scheduler_root.join("prepared-job.json"))?;
    if prepared.schema_version != IMAGE_CAMPAIGN_PREPARED_JOB_SCHEMA_V1 {
        bail!("unsupported prepared image campaign schema");
    }
    recheck_prepared(&prepared, &scheduler_root)?;
    Ok(prepared)
}

/// Load the persisted scheduler handle for one image campaign.
///
/// # Errors
/// Returns an error for missing, malformed, or mismatched evidence.
pub fn load_image_campaign_slurm_job_handle(
    spec: &ImageCampaignSlurmSpec,
) -> Result<SlurmJobHandle> {
    let prepared = load_prepared_image_campaign_slurm_job(spec)?;
    load_matching_handle(&spec.attempt_root.join("scheduler/job-handle.json"), &prepared.request)
}

/// Observe one campaign once and return immediately.
///
/// # Errors
/// Returns an error for invalid authorities or scheduler observation failure.
pub fn observe_image_campaign_slurm_job(
    paths: &SlurmClientPaths,
    spec: &ImageCampaignSlurmSpec,
    observed_at_utc: &str,
) -> Result<SlurmJobObservation> {
    validate_timestamp(observed_at_utc)?;
    observe_slurm_job(paths, &load_image_campaign_slurm_job_handle(spec)?, observed_at_utc)
}

/// Collect one campaign once, publishing immutable terminal evidence when available.
///
/// # Errors
/// Returns an error for authority drift, malformed scheduler evidence, missing terminal logs, or
/// conflicting collection evidence.
pub fn collect_image_campaign_slurm_job(
    paths: &SlurmClientPaths,
    spec: &ImageCampaignSlurmSpec,
    collected_at_utc: &str,
) -> Result<ImageCampaignCollection> {
    validate_timestamp(collected_at_utc)?;
    let prepared = load_prepared_image_campaign_slurm_job(spec)?;
    let handle = load_matching_handle(
        &spec.attempt_root.join("scheduler/job-handle.json"),
        &prepared.request,
    )?;
    let observation = collect_slurm_job(paths, &handle, collected_at_utc)?;
    if !observation.state.is_terminal() {
        return Ok(ImageCampaignCollection { observation, evidence: None });
    }
    let accounting = observation
        .accounting
        .clone()
        .context("terminal image campaign observation lacks accounting")?;
    let stdout_sha256 = require_log_digest(&handle.stdout_path, "stdout")?;
    let stderr_sha256 = require_log_digest(&handle.stderr_path, "stderr")?;
    let collection_path = spec.attempt_root.join("scheduler/collection.json");
    if regular_file_exists(&collection_path, "image campaign collection evidence")? {
        let evidence: ImageCampaignEvidence = read_json(&collection_path)?;
        validate_existing_collection(
            &evidence,
            spec,
            &prepared,
            &handle,
            &accounting,
            &stdout_sha256,
            &stderr_sha256,
        )?;
        return Ok(ImageCampaignCollection { observation, evidence: Some(evidence) });
    }
    let mut status = ImageCampaignEvidenceStatus::Failed;
    let mut failure = scheduler_failure(&observation);
    let mut completion_sha256 = None;
    let mut campaign_execution_sha256 = None;
    let mut passed_tools = None;
    let mut failed_tools = None;
    if observation.state == SlurmJobState::Completed && accounting.exit_code == 0 {
        let repository = receipt_repository(spec)?;
        match bijux_dna_containers::api::verify_apptainer_image_campaign_receipt(
            &repository,
            &spec.study_root,
            &spec.apptainer_path,
            &spec.attempt_root,
        ) {
            Ok(execution) => {
                let completion = spec.attempt_root.join("completion.json");
                let execution_path = spec.attempt_root.join("campaign-execution.json");
                completion_sha256 = Some(bijux_dna_infra::hash_file_sha256(&completion)?);
                campaign_execution_sha256 =
                    Some(bijux_dna_infra::hash_file_sha256(&execution_path)?);
                passed_tools = Some(execution.passed);
                failed_tools = Some(execution.failed);
                if execution.failed == 0 && execution.passed == spec.selected_tools.len() {
                    status = ImageCampaignEvidenceStatus::Passed;
                    failure = None;
                } else {
                    failure = Some(format!(
                        "campaign completed with {} passed and {} failed tools",
                        execution.passed, execution.failed
                    ));
                }
            }
            Err(error) => failure = Some(format!("campaign receipt admission failed: {error:#}")),
        }
    }
    let evidence = ImageCampaignEvidence {
        schema_version: IMAGE_CAMPAIGN_EVIDENCE_SCHEMA_V1.to_string(),
        status,
        run_id: spec.run_id.clone(),
        attempt_id: spec.attempt_id.clone(),
        repository_commit: spec.repository_commit.clone(),
        selected_tools: spec.selected_tools.clone(),
        job_id: handle.job_id,
        specification_sha256: prepared.specification_sha256,
        script_sha256: prepared.script_sha256,
        executable_sha256: spec.executable_sha256.clone(),
        apptainer_sha256: spec.apptainer_sha256.clone(),
        accounting,
        stdout_sha256,
        stderr_sha256,
        completion_sha256,
        campaign_execution_sha256,
        passed_tools,
        failed_tools,
        collected_at_utc: collected_at_utc.to_string(),
        failure,
    };
    ensure_json_identity(&collection_path, &evidence)?;
    Ok(ImageCampaignCollection { observation, evidence: Some(evidence) })
}

fn receipt_repository(spec: &ImageCampaignSlurmSpec) -> Result<PathBuf> {
    if bijux_dna_containers::api::clean_repository_head_commit(&spec.repository_path)
        .is_ok_and(|commit| commit == spec.repository_commit)
    {
        return Ok(spec.repository_path.clone());
    }
    let snapshot =
        spec.study_root.join("data/evidence/source-checkouts").join(&spec.repository_commit);
    bijux_dna_containers::api::verify_frozen_repository(&snapshot, &spec.repository_commit)
        .context("verify source-advanced image campaign snapshot")?;
    Ok(snapshot)
}

#[allow(clippy::too_many_arguments)]
fn validate_existing_collection(
    evidence: &ImageCampaignEvidence,
    spec: &ImageCampaignSlurmSpec,
    prepared: &PreparedImageCampaignSlurmJob,
    handle: &SlurmJobHandle,
    accounting: &crate::backend::slurm::SlurmAccounting,
    stdout_sha256: &str,
    stderr_sha256: &str,
) -> Result<()> {
    if evidence.schema_version != IMAGE_CAMPAIGN_EVIDENCE_SCHEMA_V1
        || evidence.run_id != spec.run_id
        || evidence.attempt_id != spec.attempt_id
        || evidence.repository_commit != spec.repository_commit
        || evidence.selected_tools != spec.selected_tools
        || evidence.job_id != handle.job_id
        || evidence.specification_sha256 != prepared.specification_sha256
        || evidence.script_sha256 != prepared.script_sha256
        || evidence.executable_sha256 != spec.executable_sha256
        || evidence.apptainer_sha256 != spec.apptainer_sha256
        || evidence.accounting != *accounting
        || evidence.stdout_sha256 != stdout_sha256
        || evidence.stderr_sha256 != stderr_sha256
    {
        bail!("persisted image campaign collection conflicts with current terminal evidence");
    }
    Ok(())
}

fn recheck_prepared(prepared: &PreparedImageCampaignSlurmJob, scheduler_root: &Path) -> Result<()> {
    if prepared.schema_version != IMAGE_CAMPAIGN_PREPARED_JOB_SCHEMA_V1
        || prepared.request.script_path != scheduler_root.join("job.sbatch")
        || prepared.request.stdout_path != scheduler_root.join("stdout.log")
        || prepared.request.stderr_path != scheduler_root.join("stderr.log")
        || prepared.request.array.is_some()
    {
        bail!("prepared image campaign scheduler layout is invalid");
    }
    if bijux_dna_infra::hash_file_sha256(&prepared.request.script_path)? != prepared.script_sha256 {
        bail!("prepared image campaign script digest drifted");
    }
    let specification = scheduler_root
        .parent()
        .context("image campaign scheduler root lacks attempt root")?
        .join("image-campaign-spec.json");
    if bijux_dna_infra::hash_file_sha256(&specification)? != prepared.specification_sha256 {
        bail!("prepared image campaign specification digest drifted");
    }
    Ok(())
}

fn submission_intent(prepared: &PreparedImageCampaignSlurmJob) -> SubmissionIntent {
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

fn load_matching_handle(
    path: &Path,
    request: &crate::backend::slurm::SlurmSubmitRequest,
) -> Result<SlurmJobHandle> {
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
    {
        bail!("image campaign Slurm handle conflicts with prepared request");
    }
    Ok(handle)
}

fn create_intent(path: &Path, intent: &SubmissionIntent) -> Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                bail!("image campaign submission intent is not a regular file");
            }
            if read_json::<SubmissionIntent>(path)? != *intent {
                bail!("image campaign submission intent conflicts with prepared request");
            }
            Ok(false)
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {
            bijux_dna_infra::atomic_create_json(path, intent)?;
            Ok(true)
        }
        Err(error) => Err(error).with_context(|| format!("inspect {}", path.display())),
    }
}

fn regular_file_exists(path: &Path, label: &str) -> Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => {
            bail!("{label} is not a regular non-symlink file")
        }
        Ok(_) => Ok(true),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error).with_context(|| format!("inspect {label}")),
    }
}

fn scheduler_root(script_path: &Path) -> Result<PathBuf> {
    let root = script_path.parent().context("image campaign script lacks scheduler root")?;
    canonical_directory(root, "image campaign scheduler root")
}

fn require_log_digest(path: &Path, label: &str) -> Result<String> {
    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("inspect image campaign {label} log"))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!("image campaign {label} log is not a regular file");
    }
    bijux_dna_infra::hash_file_sha256(path).with_context(|| format!("hash {label} log"))
}

fn scheduler_failure(observation: &SlurmJobObservation) -> Option<String> {
    let accounting = observation.accounting.as_ref()?;
    (observation.state != SlurmJobState::Completed || accounting.exit_code != 0).then(|| {
        format!(
            "scheduler state {:?} exited {}:{}",
            observation.state, accounting.exit_code, accounting.signal
        )
    })
}

fn validate_timestamp(value: &str) -> Result<()> {
    let parsed = chrono::DateTime::parse_from_rfc3339(value)
        .context("image campaign observation timestamp is invalid")?;
    if parsed.offset().local_minus_utc() != 0 || !value.ends_with('Z') {
        bail!("image campaign observation timestamp must use UTC Z notation");
    }
    Ok(())
}
