//! Owner: bijux-dna-runner
//! Restart-safe non-waiting Slurm lifecycle for FASTQ corpus acquisition.

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{CorpusAcquisitionPlan, GovernedDataCatalog};
use serde::{Deserialize, Serialize};

use super::contracts::{
    CorpusMaterializationCollection, CorpusMaterializationEvidence,
    CorpusMaterializationEvidenceStatus, CorpusMaterializationSlurmSpec,
    PreparedCorpusMaterializationSlurmJob, CORPUS_MATERIALIZATION_EVIDENCE_SCHEMA_V1,
    CORPUS_MATERIALIZATION_PREPARED_JOB_SCHEMA_V1,
};
use super::validation::{
    canonical_directory, ensure_json_identity, read_json, require_utc_timestamp,
    validate_catalog_closure, validate_specification,
};
use crate::backend::slurm::{
    collect_slurm_job, observe_slurm_job, submit_slurm_job, SlurmArraySpec, SlurmClientPaths,
    SlurmDependencyCondition, SlurmJobHandle, SlurmJobObservation, SlurmJobState,
    SlurmSubmitRequest, SLURM_JOB_HANDLE_SCHEMA_V1,
};

const SUBMISSION_INTENT_SCHEMA_V1: &str =
    "bijux.runner.corpus_materialization_submission_intent.v1";
const SUBMISSION_FAILURE_SCHEMA_V1: &str =
    "bijux.runner.corpus_materialization_submission_failure.v1";

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

#[derive(Debug, Deserialize)]
struct PublicationReceipt {
    catalog_path: PathBuf,
    catalog_sha256: String,
    catalog: GovernedDataCatalog,
}

#[derive(Debug, Deserialize)]
struct CompletionReceipt {
    schema_version: String,
    specification_sha256: String,
}

/// Submit a prepared corpus materialization job once after persisting immutable intent.
///
/// # Errors
/// Returns an error for material drift, unresolved prior intent, conflicting evidence, scheduler
/// rejection, or failure to persist the returned handle.
pub fn submit_prepared_corpus_materialization_job(
    paths: &SlurmClientPaths,
    prepared: &PreparedCorpusMaterializationSlurmJob,
) -> Result<SlurmJobHandle> {
    let scheduler_root = scheduler_root(&prepared.request)?;
    recheck_prepared(prepared, &scheduler_root)?;
    let persisted_prepared: PreparedCorpusMaterializationSlurmJob =
        read_json(&scheduler_root.join("prepared-job.json"))?;
    if persisted_prepared != *prepared {
        bail!("prepared corpus materialization job conflicts with immutable scheduler evidence");
    }
    let intent = submission_intent(prepared);
    let intent_path = scheduler_root.join("submission-intent.json");
    let handle_path = scheduler_root.join("job-handle.json");
    if regular_file_exists(&handle_path, "corpus materialization Slurm handle")? {
        let persisted_intent: SubmissionIntent = read_json(&intent_path)?;
        if persisted_intent != intent {
            bail!("corpus materialization submission intent conflicts with persisted handle");
        }
        return load_matching_handle(&handle_path, &prepared.request);
    }
    if !create_intent(&intent_path, &intent)? {
        bail!(
            "corpus materialization submission intent has no handle; reconcile scheduler state before retry"
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
            return Err(error).context("submit prepared corpus materialization job");
        }
    };
    ensure_json_identity(&handle_path, &handle)?;
    Ok(handle)
}

/// Load the immutable prepared job after revalidating its specification and scheduler material.
///
/// # Errors
/// Returns an error for missing, changed, or conflicting preparation evidence.
pub fn load_prepared_corpus_materialization_slurm_job(
    spec: &CorpusMaterializationSlurmSpec,
) -> Result<PreparedCorpusMaterializationSlurmJob> {
    validate_specification(spec)?;
    let evidence_root =
        canonical_directory(&spec.evidence_root, "corpus materialization evidence root")?;
    let persisted_spec: CorpusMaterializationSlurmSpec =
        read_json(&evidence_root.join("materialization-spec.json"))?;
    if persisted_spec != *spec {
        bail!("persisted materialization specification conflicts with requested identity");
    }
    let scheduler_root =
        canonical_directory(&evidence_root.join("scheduler"), "materialization scheduler root")?;
    let prepared: PreparedCorpusMaterializationSlurmJob =
        read_json(&scheduler_root.join("prepared-job.json"))?;
    require_utc_timestamp(&prepared.request.submitted_at_utc, "submission timestamp")?;
    if prepared.schema_version != CORPUS_MATERIALIZATION_PREPARED_JOB_SCHEMA_V1
        || prepared.request.run_id
            != format!("{}:{}:{}", spec.run_id, spec.portfolio_id, spec.attempt_id)
        || prepared.request.script_path != scheduler_root.join("job.sbatch")
        || prepared.request.working_directory != spec.repository_path
        || prepared.request.stdout_path != scheduler_root.join("stdout.log")
        || prepared.request.stderr_path != scheduler_root.join("stderr.log")
    {
        bail!("prepared materialization job conflicts with requested specification");
    }
    recheck_prepared(&prepared, &scheduler_root)?;
    Ok(prepared)
}

/// Load the persisted scheduler handle after revalidating specification and script identity.
///
/// # Errors
/// Returns an error for missing, changed, or conflicting scheduler evidence.
pub fn load_corpus_materialization_slurm_job_handle(
    spec: &CorpusMaterializationSlurmSpec,
) -> Result<SlurmJobHandle> {
    let prepared = load_prepared_corpus_materialization_slurm_job(spec)?;
    let scheduler_root = scheduler_root(&prepared.request)?;
    let intent: SubmissionIntent = read_json(&scheduler_root.join("submission-intent.json"))?;
    if submission_intent(&prepared) != intent {
        bail!("persisted materialization submission intent conflicts with scheduler material");
    }
    load_matching_handle(&scheduler_root.join("job-handle.json"), &prepared.request)
}

/// Observe one materialization job once without waiting or mutating scheduler evidence.
///
/// # Errors
/// Returns an error when retained identity or scheduler output is invalid.
pub fn observe_corpus_materialization_slurm_job(
    paths: &SlurmClientPaths,
    spec: &CorpusMaterializationSlurmSpec,
    observed_at_utc: &str,
) -> Result<SlurmJobObservation> {
    require_utc_timestamp(observed_at_utc, "observation timestamp")?;
    let handle = load_corpus_materialization_slurm_job_handle(spec)?;
    observe_slurm_job(paths, &handle, observed_at_utc)
}

/// Collect one scheduler observation and publish immutable terminal materialization evidence.
///
/// Active and accounting-pending jobs return immediately without publication. A retained terminal
/// publication is returned idempotently without another scheduler call.
///
/// # Errors
/// Returns an error for invalid scheduler evidence, changed authorities, malformed catalog
/// closure, or conflicting terminal publication.
pub fn collect_corpus_materialization_slurm_job(
    paths: &SlurmClientPaths,
    spec: &CorpusMaterializationSlurmSpec,
    collected_at_utc: &str,
) -> Result<CorpusMaterializationCollection> {
    require_utc_timestamp(collected_at_utc, "collection timestamp")?;
    let plan = validate_specification(spec)?;
    let handle = load_corpus_materialization_slurm_job_handle(spec)?;
    let terminal_path = spec.evidence_root.join("materialization-evidence.json");
    if regular_file_exists(&terminal_path, "terminal materialization evidence")? {
        let evidence: CorpusMaterializationEvidence = read_json(&terminal_path)?;
        validate_retained_evidence(spec, &plan, &handle, &evidence)?;
        let observation = SlurmJobObservation {
            job_id: evidence.job_id.clone(),
            state: evidence.accounting.state,
            source: crate::backend::slurm::SlurmEvidenceSource::Accounting,
            observed_at_utc: evidence.collected_at_utc.clone(),
            accounting: Some(evidence.accounting.clone()),
        };
        return Ok(CorpusMaterializationCollection { observation, evidence: Some(evidence) });
    }

    let observation = collect_slurm_job(paths, &handle, collected_at_utc)?;
    if !observation.state.is_terminal() {
        return Ok(CorpusMaterializationCollection { observation, evidence: None });
    }
    let accounting = observation
        .accounting
        .clone()
        .context("terminal materialization observation lacks accounting")?;
    let prepared = prepared_from_evidence(spec, &handle)?;
    let scheduler_root = spec.evidence_root.join("scheduler");
    let stdout_sha256 = hash_required_or_empty(&scheduler_root.join("stdout.log"), "stdout")?;
    let stderr_sha256 = hash_required_or_empty(&scheduler_root.join("stderr.log"), "stderr")?;
    let publication_path = spec.evidence_root.join("materialization-publication.json");

    let mut status = CorpusMaterializationEvidenceStatus::Failed;
    let mut publication_sha256 = None;
    let mut catalog_path = None;
    let mut catalog_sha256 = None;
    let mut failure = Some(terminal_failure(&accounting));
    if accounting.state == SlurmJobState::Completed && accounting.exit_code == 0 {
        match validate_success(spec, &plan, &publication_path) {
            Ok((observed_catalog_path, observed_catalog_sha256, observed_publication_sha256)) => {
                status = CorpusMaterializationEvidenceStatus::Passed;
                publication_sha256 = Some(observed_publication_sha256);
                catalog_path = Some(observed_catalog_path);
                catalog_sha256 = Some(observed_catalog_sha256);
                failure = None;
            }
            Err(error) => failure = Some(format!("{error:#}")),
        }
    }
    let evidence = CorpusMaterializationEvidence {
        schema_version: CORPUS_MATERIALIZATION_EVIDENCE_SCHEMA_V1.to_string(),
        status,
        run_id: spec.run_id.clone(),
        attempt_id: spec.attempt_id.clone(),
        portfolio_id: spec.portfolio_id.clone(),
        repository_commit: spec.repository_commit.clone(),
        job_id: handle.job_id,
        specification_sha256: prepared.specification_sha256,
        script_sha256: prepared.script_sha256,
        executable_sha256: spec.executable_sha256.clone(),
        plan_sha256: spec.plan_sha256.clone(),
        accounting,
        stdout_sha256,
        stderr_sha256,
        publication_sha256,
        catalog_path,
        catalog_sha256,
        collected_at_utc: collected_at_utc.to_string(),
        failure,
    };
    ensure_json_identity(&terminal_path, &evidence)?;
    Ok(CorpusMaterializationCollection { observation, evidence: Some(evidence) })
}

fn validate_success(
    spec: &CorpusMaterializationSlurmSpec,
    plan: &CorpusAcquisitionPlan,
    publication_path: &Path,
) -> Result<(PathBuf, String, String)> {
    let specification_sha256 =
        bijux_dna_infra::hash_file_sha256(&spec.evidence_root.join("materialization-spec.json"))?;
    let completion: CompletionReceipt =
        read_json(&spec.evidence_root.join("scheduler/completed.json"))?;
    if completion.schema_version != "bijux.runner.corpus_materialization_completion.v1"
        || completion.specification_sha256 != specification_sha256
    {
        bail!("materialization completion receipt differs from scheduler specification");
    }
    let (catalog_path, catalog_sha256) = validate_catalog_closure(spec, plan)?;
    let catalog: GovernedDataCatalog = read_json(&catalog_path)?;
    let publication: PublicationReceipt = read_json(publication_path)?;
    if publication.catalog_path != catalog_path
        || publication.catalog_sha256 != catalog_sha256
        || publication.catalog != catalog
    {
        bail!("materialization publication differs from verified governed catalog");
    }
    let publication_sha256 = bijux_dna_infra::hash_file_sha256(publication_path)
        .context("hash materialization publication")?;
    Ok((catalog_path, catalog_sha256, publication_sha256))
}

fn validate_retained_evidence(
    spec: &CorpusMaterializationSlurmSpec,
    plan: &CorpusAcquisitionPlan,
    handle: &SlurmJobHandle,
    evidence: &CorpusMaterializationEvidence,
) -> Result<()> {
    let prepared = prepared_from_evidence(spec, handle)?;
    if evidence.schema_version != CORPUS_MATERIALIZATION_EVIDENCE_SCHEMA_V1
        || evidence.run_id != spec.run_id
        || evidence.attempt_id != spec.attempt_id
        || evidence.portfolio_id != spec.portfolio_id
        || evidence.repository_commit != spec.repository_commit
        || evidence.job_id != handle.job_id
        || evidence.specification_sha256 != prepared.specification_sha256
        || evidence.script_sha256 != prepared.script_sha256
        || evidence.executable_sha256 != spec.executable_sha256
        || evidence.plan_sha256 != spec.plan_sha256
        || evidence.accounting.job_id != handle.job_id
        || !evidence.accounting.state.is_terminal()
    {
        bail!("retained materialization evidence conflicts with requested identity");
    }
    require_utc_timestamp(&evidence.collected_at_utc, "retained collection timestamp")?;
    let scheduler_root = spec.evidence_root.join("scheduler");
    let stdout_sha256 = hash_required_or_empty(&scheduler_root.join("stdout.log"), "stdout")?;
    let stderr_sha256 = hash_required_or_empty(&scheduler_root.join("stderr.log"), "stderr")?;
    if evidence.stdout_sha256 != stdout_sha256 || evidence.stderr_sha256 != stderr_sha256 {
        bail!("retained materialization evidence conflicts with scheduler log digests");
    }
    if evidence.status == CorpusMaterializationEvidenceStatus::Passed {
        let (catalog_path, catalog_sha256, publication_sha256) = validate_success(
            spec,
            plan,
            &spec.evidence_root.join("materialization-publication.json"),
        )?;
        if evidence.catalog_path.as_ref() != Some(&catalog_path)
            || evidence.catalog_sha256.as_deref() != Some(catalog_sha256.as_str())
            || evidence.publication_sha256.as_deref() != Some(publication_sha256.as_str())
            || evidence.accounting.state != SlurmJobState::Completed
            || evidence.accounting.exit_code != 0
            || evidence.accounting.signal != 0
            || evidence.failure.is_some()
        {
            bail!("retained successful materialization evidence conflicts with catalog closure");
        }
    } else if evidence.publication_sha256.is_some()
        || evidence.catalog_path.is_some()
        || evidence.catalog_sha256.is_some()
        || evidence.failure.as_deref().is_none_or(str::is_empty)
    {
        bail!("retained failed materialization evidence has inconsistent terminal fields");
    }
    Ok(())
}

fn prepared_from_evidence(
    spec: &CorpusMaterializationSlurmSpec,
    handle: &SlurmJobHandle,
) -> Result<PreparedCorpusMaterializationSlurmJob> {
    Ok(PreparedCorpusMaterializationSlurmJob {
        schema_version: CORPUS_MATERIALIZATION_PREPARED_JOB_SCHEMA_V1.to_string(),
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
            &spec.evidence_root.join("materialization-spec.json"),
        )?,
        script_sha256: bijux_dna_infra::hash_file_sha256(
            &spec.evidence_root.join("scheduler/job.sbatch"),
        )?,
    })
}

fn scheduler_root(request: &SlurmSubmitRequest) -> Result<PathBuf> {
    let path = request.script_path.canonicalize().context("canonicalize materialization script")?;
    if path.file_name().and_then(|name| name.to_str()) != Some("job.sbatch") {
        bail!("prepared corpus materialization script has an unsupported name");
    }
    canonical_directory(
        path.parent().context("materialization script has no parent")?,
        "materialization scheduler root",
    )
}

fn recheck_prepared(
    prepared: &PreparedCorpusMaterializationSlurmJob,
    scheduler_root: &Path,
) -> Result<()> {
    let script_sha256 = bijux_dna_infra::hash_file_sha256(&prepared.request.script_path)?;
    let specification_sha256 = bijux_dna_infra::hash_file_sha256(
        &scheduler_root
            .parent()
            .context("scheduler root has no evidence parent")?
            .join("materialization-spec.json"),
    )?;
    if script_sha256 != prepared.script_sha256
        || specification_sha256 != prepared.specification_sha256
    {
        bail!("prepared corpus materialization scheduler material changed");
    }
    Ok(())
}

fn submission_intent(prepared: &PreparedCorpusMaterializationSlurmJob) -> SubmissionIntent {
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
                bail!("materialization submission intent is not a regular file");
            }
            let existing: SubmissionIntent = read_json(path)?;
            if existing != *intent {
                bail!("materialization submission intent conflicts with prepared material");
            }
            Ok(false)
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {
            bijux_dna_infra::atomic_create_json(path, intent)?;
            Ok(true)
        }
        Err(error) => Err(error).context("inspect materialization submission intent"),
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
        bail!("persisted corpus materialization Slurm handle conflicts with prepared material");
    }
    Ok(handle)
}

fn regular_file_exists(path: &Path, label: &str) -> Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_file() && !metadata.file_type().is_symlink() => Ok(true),
        Ok(_) => bail!("{label} is not a regular file"),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error).with_context(|| format!("inspect {label} {}", path.display())),
    }
}

fn hash_required_or_empty(path: &Path, label: &str) -> Result<String> {
    if !regular_file_exists(path, &format!("materialization {label} log"))? {
        bail!("materialization {label} log is missing");
    }
    bijux_dna_infra::hash_file_sha256(path).with_context(|| format!("hash {label} log"))
}

fn terminal_failure(accounting: &crate::backend::slurm::SlurmAccounting) -> String {
    format!(
        "Slurm terminal state {:?} with exit code {} and signal {}",
        accounting.state, accounting.exit_code, accounting.signal
    )
}
