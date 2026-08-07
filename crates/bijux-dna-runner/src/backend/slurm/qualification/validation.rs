//! Owner: bijux-dna-runner
//! Identity validation for container qualification scheduler jobs.

use std::ffi::OsStr;
use std::fs;
use std::path::{Component, Path, PathBuf};

use anyhow::{bail, Context, Result};
use bijux_dna_containers::api::{
    admit_container_pipeline_site, admit_historical_container_pipeline_replay,
    load_container_credential_reference, load_container_governance, load_container_pipeline,
    ContainerPipelineContract, ContainerPipelineSiteAdmission,
    ContainerPipelineSiteAdmissionRequest, HistoricalContainerPipelineReplayRequest,
    PipelineEvidenceKind,
};
use serde::de::DeserializeOwned;

use super::contracts::{
    ContainerQualificationSlurmResources, ContainerQualificationSlurmSpec,
    ContainerQualificationSubmissionKind, CONTAINER_QUALIFICATION_SLURM_SPEC_SCHEMA_V1,
    CONTAINER_QUALIFICATION_SLURM_SPEC_SCHEMA_V2, CONTAINER_QUALIFICATION_SLURM_SPEC_SCHEMA_V3,
    CONTAINER_QUALIFICATION_SLURM_SPEC_SCHEMA_V4,
};

pub(super) const MAX_EVIDENCE_BYTES: usize = 16 * 1024 * 1024;

pub(super) struct QualificationIdentity {
    pub run_id: String,
    pub tool_id: String,
    pub job_name: String,
    pub repository_path: PathBuf,
    pub repository_commit: String,
    pub result_root: PathBuf,
    pub evidence_root: PathBuf,
    pub resources: ContainerQualificationSlurmResources,
    pub dependencies: Vec<String>,
}

/// Load and validate one persisted container qualification scheduler specification.
///
/// # Errors
/// Returns an error when the file is unsafe, malformed, changed, or conflicts with its admitted
/// governance, pipeline, credential, executor, repository, or evidence-root identities.
pub fn load_container_qualification_slurm_spec(
    path: &Path,
) -> Result<ContainerQualificationSlurmSpec> {
    let spec = read_json(path)?;
    validate_specification(&spec)?;
    Ok(spec)
}

pub(super) fn derive_identity(
    hostname: &str,
    governance_path: &Path,
    pipeline_path: &Path,
    attempt_id: Option<&str>,
) -> Result<QualificationIdentity> {
    let governance = load_container_governance(governance_path)?;
    let contract = load_container_pipeline(pipeline_path)?;
    let admission = admit_container_pipeline_site(&ContainerPipelineSiteAdmissionRequest {
        governance: governance.clone(),
        hostname: hostname.to_string(),
        pipeline_path: pipeline_path.to_path_buf(),
    })?;
    derive_admitted_identity(&governance, contract, admission, false, attempt_id)
}

pub(super) fn derive_historical_replay_identity(
    hostname: &str,
    governance_path: &Path,
    source_pipeline_path: &Path,
    replay_pipeline_path: &Path,
    attempt_id: Option<&str>,
) -> Result<QualificationIdentity> {
    let governance = load_container_governance(governance_path)?;
    let replay =
        admit_historical_container_pipeline_replay(&HistoricalContainerPipelineReplayRequest {
            governance: governance.clone(),
            hostname: hostname.to_string(),
            source_pipeline_path: source_pipeline_path.to_path_buf(),
            replay_pipeline_path: replay_pipeline_path.to_path_buf(),
        })?;
    let contract = load_container_pipeline(replay_pipeline_path)?;
    derive_admitted_identity(&governance, contract, replay.site, true, attempt_id)
}

fn derive_admitted_identity(
    governance: &bijux_dna_containers::api::ContainerGovernance,
    contract: ContainerPipelineContract,
    admission: ContainerPipelineSiteAdmission,
    historical_replay: bool,
    attempt_id: Option<&str>,
) -> Result<QualificationIdentity> {
    if admission.evidence_kind != PipelineEvidenceKind::Qualification {
        bail!("container qualification scheduler requires qualification evidence kind");
    }
    if admission.repository_path == governance.paths.active_repository {
        bail!("container qualification requires a commit-addressed frozen repository");
    }
    let frozen_root =
        canonical_directory(&governance.paths.frozen_repositories, "frozen repository root")?;
    if admission.repository_path.parent() != Some(frozen_root.as_path())
        || admission.repository_path.file_name().and_then(|name| name.to_str())
            != Some(admission.repository_commit.as_str())
    {
        bail!("container qualification repository is not frozen by commit identity");
    }

    let (run_id, tool_id, domain, stage, sample, resources, dependencies) = match contract {
        ContainerPipelineContract::Legacy(pipeline) if historical_replay => (
            pipeline.run.id,
            pipeline.run.tool,
            pipeline.run.domain,
            pipeline.run.stage,
            pipeline.run.sample,
            ContainerQualificationSlurmResources {
                account: pipeline.resources.account,
                partition: pipeline.resources.partition,
                cpus: pipeline.resources.cpus,
                memory_mb: pipeline.resources.memory_mb,
                walltime: pipeline.resources.walltime,
                scratch_mb: pipeline.resources.scratch_mb,
            },
            Vec::new(),
        ),
        ContainerPipelineContract::Legacy(_) => {
            bail!("new container qualification submission requires the current pipeline schema")
        }
        ContainerPipelineContract::Current(pipeline) => (
            pipeline.run.id,
            pipeline.run.tool,
            pipeline.run.domain,
            pipeline.run.stage,
            pipeline.run.sample,
            ContainerQualificationSlurmResources {
                account: pipeline.resources.account,
                partition: pipeline.resources.partition,
                cpus: pipeline.resources.cpus,
                memory_mb: pipeline.resources.memory_mb,
                walltime: pipeline.resources.walltime,
                scratch_mb: pipeline.resources.scratch_mb,
            },
            pipeline.dependencies.into_iter().map(|dependency| dependency.slurm_job_id).collect(),
        ),
    };
    let evidence_namespace = canonical_governed_directory(
        &governance.paths.containers_root,
        "governed containers root",
    )?
    .join("evidence/submissions")
    .join(format!("{run_id}-{tool_id}"));
    let (result_root, evidence_root) = match attempt_id {
        Some(attempt_id) => {
            validate_qualification_attempt_id(attempt_id)?;
            (
                admission.result_root.join("attempts").join(attempt_id),
                evidence_namespace.join("attempts").join(attempt_id),
            )
        }
        None => (admission.result_root, evidence_namespace),
    };
    Ok(QualificationIdentity {
        job_name: scheduler_name(&domain, &stage, &tool_id, &sample)?,
        run_id,
        tool_id,
        repository_path: admission.repository_path,
        repository_commit: admission.repository_commit,
        result_root,
        evidence_root,
        resources,
        dependencies,
    })
}

pub(super) fn validate_specification(spec: &ContainerQualificationSlurmSpec) -> Result<()> {
    validate_schema_authority(spec)?;
    require_sha256(&spec.governance_sha256, "governance SHA-256")?;
    require_sha256(&spec.pipeline_sha256, "pipeline SHA-256")?;
    require_sha256(&spec.credential_reference_sha256, "credential-reference SHA-256")?;
    require_sha256(&spec.executor_program_sha256, "executor program SHA-256")?;
    match (&spec.submission_kind, &spec.historical_replay_source) {
        (ContainerQualificationSubmissionKind::Current, None) => {}
        (ContainerQualificationSubmissionKind::HistoricalReplay, Some(source)) => {
            require_sha256(&source.pipeline_sha256, "historical source pipeline SHA-256")?;
            require_exact_file(
                &source.pipeline_path,
                &source.pipeline_sha256,
                "historical source pipeline",
            )?;
        }
        _ => bail!("qualification submission kind conflicts with historical replay authority"),
    }
    require_exact_file(&spec.governance_path, &spec.governance_sha256, "governance")?;
    require_exact_file(&spec.pipeline_path, &spec.pipeline_sha256, "pipeline")?;
    require_exact_file(
        &spec.credential_reference_path,
        &spec.credential_reference_sha256,
        "credential reference",
    )?;
    require_exact_file(
        &spec.executor_program_path,
        &spec.executor_program_sha256,
        "executor program",
    )?;
    require_executable(&spec.executor_program_path, "executor program")?;
    if matches!(
        spec.schema_version.as_str(),
        CONTAINER_QUALIFICATION_SLURM_SPEC_SCHEMA_V3 | CONTAINER_QUALIFICATION_SLURM_SPEC_SCHEMA_V4
    ) && spec.executor_program_path.file_name() != Some(OsStr::new("bijux-dna-dev"))
    {
        bail!("native qualification executor must be the pinned bijux-dna-dev program");
    }
    if let (Some(path), Some(sha256)) =
        (spec.executor_script_path.as_ref(), spec.executor_script_sha256.as_deref())
    {
        require_exact_file(path, sha256, "executor script")?;
    }
    load_container_credential_reference(&spec.credential_reference_path)?;
    let identity = match &spec.historical_replay_source {
        Some(source) => derive_historical_replay_identity(
            &spec.hostname,
            &spec.governance_path,
            &source.pipeline_path,
            &spec.pipeline_path,
            spec.attempt_id.as_deref(),
        )?,
        None => derive_identity(
            &spec.hostname,
            &spec.governance_path,
            &spec.pipeline_path,
            spec.attempt_id.as_deref(),
        )?,
    };
    if spec.run_id != identity.run_id
        || spec.tool_id != identity.tool_id
        || spec.job_name != identity.job_name
        || spec.repository_path != identity.repository_path
        || spec.repository_commit != identity.repository_commit
        || spec.result_root != identity.result_root
        || spec.evidence_root != identity.evidence_root
        || spec.resources != identity.resources
        || spec.dependencies != identity.dependencies
    {
        bail!("container qualification specification conflicts with admitted contracts");
    }
    if !matches!(
        spec.schema_version.as_str(),
        CONTAINER_QUALIFICATION_SLURM_SPEC_SCHEMA_V3 | CONTAINER_QUALIFICATION_SLURM_SPEC_SCHEMA_V4
    ) {
        let expected_executor =
            spec.repository_path.join("containers/contracts/container_validation.py");
        if spec.executor_script_path.as_deref() != Some(expected_executor.as_path()) {
            bail!("container qualification executor is outside its frozen repository authority");
        }
    }
    canonical_directory(&spec.evidence_root, "qualification evidence root")?;
    Ok(())
}

fn validate_schema_authority(spec: &ContainerQualificationSlurmSpec) -> Result<()> {
    if !matches!(
        spec.schema_version.as_str(),
        CONTAINER_QUALIFICATION_SLURM_SPEC_SCHEMA_V1
            | CONTAINER_QUALIFICATION_SLURM_SPEC_SCHEMA_V2
            | CONTAINER_QUALIFICATION_SLURM_SPEC_SCHEMA_V3
            | CONTAINER_QUALIFICATION_SLURM_SPEC_SCHEMA_V4
    ) {
        bail!("container qualification Slurm specification schema is unsupported");
    }
    match (spec.schema_version.as_str(), spec.attempt_id.as_deref()) {
        (CONTAINER_QUALIFICATION_SLURM_SPEC_SCHEMA_V4, Some(attempt_id)) => {
            validate_qualification_attempt_id(attempt_id)?;
        }
        (
            CONTAINER_QUALIFICATION_SLURM_SPEC_SCHEMA_V1
            | CONTAINER_QUALIFICATION_SLURM_SPEC_SCHEMA_V2
            | CONTAINER_QUALIFICATION_SLURM_SPEC_SCHEMA_V3,
            None,
        ) => {}
        _ => bail!("qualification attempt identity conflicts with specification schema"),
    }
    if spec.schema_version == CONTAINER_QUALIFICATION_SLURM_SPEC_SCHEMA_V1
        && (spec.submission_kind != ContainerQualificationSubmissionKind::Current
            || spec.historical_replay_source.is_some())
    {
        bail!("v1 qualification specification cannot declare historical replay authority");
    }
    match (
        spec.schema_version.as_str(),
        spec.executor_script_path.as_ref(),
        spec.executor_script_sha256.as_deref(),
    ) {
        (
            CONTAINER_QUALIFICATION_SLURM_SPEC_SCHEMA_V3
            | CONTAINER_QUALIFICATION_SLURM_SPEC_SCHEMA_V4,
            None,
            None,
        ) => {}
        (
            CONTAINER_QUALIFICATION_SLURM_SPEC_SCHEMA_V1
            | CONTAINER_QUALIFICATION_SLURM_SPEC_SCHEMA_V2,
            Some(_),
            Some(sha256),
        ) => require_sha256(sha256, "executor script SHA-256")?,
        _ => bail!("qualification executor authority conflicts with specification schema"),
    }
    Ok(())
}

fn validate_qualification_attempt_id(attempt_id: &str) -> Result<()> {
    let Some(sequence) = attempt_id.strip_prefix("attempt-") else {
        bail!("qualification attempt identity must use attempt-NNN form");
    };
    if sequence.len() < 3
        || !sequence.bytes().all(|byte| byte.is_ascii_digit())
        || sequence.bytes().all(|byte| byte == b'0')
    {
        bail!("qualification attempt identity must use a nonzero numeric sequence");
    }
    Ok(())
}

pub(super) fn read_json<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("inspect JSON authority {}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!("JSON authority is not a regular file: {}", path.display());
    }
    let bytes = bijux_dna_infra::read_to_end_bounded(path, MAX_EVIDENCE_BYTES)?;
    serde_json::from_slice(&bytes).with_context(|| format!("parse {}", path.display()))
}

pub(super) fn ensure_json_identity<T>(path: &Path, value: &T) -> Result<()>
where
    T: serde::Serialize + DeserializeOwned + PartialEq,
{
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                bail!("immutable JSON evidence is not a regular file: {}", path.display());
            }
            if read_json::<T>(path)? != *value {
                bail!("immutable JSON evidence conflicts at {}", path.display());
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            bijux_dna_infra::atomic_create_json(path, value)?;
        }
        Err(error) => return Err(error).with_context(|| format!("inspect {}", path.display())),
    }
    Ok(())
}

pub(super) fn require_utc_timestamp(value: &str, label: &str) -> Result<()> {
    bijux_dna_bench_model::validate_utc_timestamp(value, label).map_err(Into::into)
}

pub(super) fn canonical_directory(path: &Path, label: &str) -> Result<PathBuf> {
    let metadata = fs::symlink_metadata(path).with_context(|| format!("inspect {label}"))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        bail!("{label} is not a regular directory");
    }
    let canonical = path.canonicalize().with_context(|| format!("canonicalize {label}"))?;
    if canonical != path {
        bail!("{label} is not canonical");
    }
    Ok(canonical)
}

fn canonical_governed_directory(path: &Path, label: &str) -> Result<PathBuf> {
    if !path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, Component::ParentDir | Component::CurDir))
    {
        bail!("{label} path is not absolute and normalized");
    }
    let metadata = fs::symlink_metadata(path).with_context(|| format!("inspect {label}"))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        bail!("{label} is not a regular directory");
    }
    path.canonicalize().with_context(|| format!("canonicalize {label}"))
}

pub(super) fn require_exact_file(path: &Path, sha256: &str, label: &str) -> Result<PathBuf> {
    if !path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, Component::ParentDir | Component::CurDir))
    {
        bail!("{label} path is not absolute and normalized");
    }
    let metadata = fs::symlink_metadata(path).with_context(|| format!("inspect {label}"))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!("{label} is not a regular file");
    }
    let canonical = path.canonicalize().with_context(|| format!("canonicalize {label}"))?;
    if canonical != path {
        bail!("{label} path is not canonical");
    }
    let observed = bijux_dna_infra::hash_file_sha256(path)?;
    if observed != sha256 {
        bail!("{label} digest differs from qualification specification");
    }
    Ok(canonical)
}

fn scheduler_name(domain: &str, stage: &str, tool: &str, sample: &str) -> Result<String> {
    let stage = stage.rsplit('.').next().context("pipeline stage is empty")?;
    let name = format!("{domain}-{stage}-{tool}-{sample}");
    if name.is_empty()
        || name.len() > 128
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        bail!("container qualification job name is not scheduler-safe");
    }
    Ok(name)
}

fn require_sha256(value: &str, label: &str) -> Result<()> {
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        bail!("{label} is malformed");
    }
    Ok(())
}

fn require_executable(path: &Path, label: &str) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        if fs::metadata(path)?.permissions().mode() & 0o111 == 0 {
            bail!("{label} is not executable");
        }
    }
    Ok(())
}
