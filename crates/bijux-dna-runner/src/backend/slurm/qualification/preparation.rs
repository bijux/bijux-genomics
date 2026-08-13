//! Owner: bijux-dna-runner
//! Immutable scheduler material for governed container qualification.

use std::ffi::OsStr;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use bijux_dna_containers::api::{load_container_credential_reference, load_container_governance};

use super::contracts::{
    ContainerQualificationPreparationRequest, ContainerQualificationSlurmSpec,
    ContainerQualificationSubmissionKind, HistoricalContainerQualificationReplayRequest,
    HistoricalContainerQualificationReplaySource, PreparedContainerQualificationSlurmJob,
    CONTAINER_QUALIFICATION_PREPARED_JOB_SCHEMA_V1, CONTAINER_QUALIFICATION_SLURM_SPEC_SCHEMA_V5,
};
use super::validation::{
    canonical_directory, canonical_governed_directory, derive_historical_replay_identity,
    derive_identity, ensure_json_identity, require_exact_file, require_executable,
    require_utc_timestamp, validate_specification, QualificationIdentity,
};
use crate::backend::slurm::{SlurmDependencyCondition, SlurmSubmitRequest};

/// Archive declarative inputs and prepare one restart-safe qualification scheduler job.
///
/// # Errors
/// Returns an error when site admission fails, the repository is not frozen, source identities
/// drift, retained evidence conflicts, or scheduler material cannot be published atomically.
pub fn prepare_container_qualification_slurm_job(
    request: &ContainerQualificationPreparationRequest,
    submitted_at_utc: &str,
) -> Result<PreparedContainerQualificationSlurmJob> {
    prepare(request, None, ContainerQualificationSubmissionKind::Current, submitted_at_utc)
}

/// Prepare immutable scheduler material for one exact historical v1 protocol replay.
///
/// # Errors
/// Returns an error when replay admission fails, immutable identities drift, or scheduler material
/// cannot be published atomically.
pub fn prepare_historical_container_qualification_replay(
    request: &HistoricalContainerQualificationReplayRequest,
    submitted_at_utc: &str,
) -> Result<PreparedContainerQualificationSlurmJob> {
    let preparation = ContainerQualificationPreparationRequest {
        attempt_id: request.attempt_id.clone(),
        hostname: request.hostname.clone(),
        governance_path: request.governance_path.clone(),
        pipeline_path: request.replay_pipeline_path.clone(),
        credential_reference_path: request.credential_reference_path.clone(),
        executor_program_path: request.executor_program_path.clone(),
    };
    prepare(
        &preparation,
        Some(&request.source_pipeline_path),
        ContainerQualificationSubmissionKind::HistoricalReplay,
        submitted_at_utc,
    )
}

fn prepare(
    request: &ContainerQualificationPreparationRequest,
    historical_source_pipeline: Option<&Path>,
    submission_kind: ContainerQualificationSubmissionKind,
    submitted_at_utc: &str,
) -> Result<PreparedContainerQualificationSlurmJob> {
    require_utc_timestamp(submitted_at_utc, "submission timestamp")?;
    load_container_credential_reference(&request.credential_reference_path)?;
    let identity = derive_preparation_identity(request, historical_source_pipeline)?;
    let executor_program_path = request
        .executor_program_path
        .canonicalize()
        .context("canonicalize qualification executor program")?;
    if executor_program_path.file_name() != Some(OsStr::new("bijux-dna-dev")) {
        bail!("native qualification executor must be the pinned bijux-dna-dev program");
    }
    let governance = load_container_governance(&request.governance_path)?;
    let executor_program_sha256 = bijux_dna_infra::hash_file_sha256(&executor_program_path)?;
    require_exact_file(&executor_program_path, &executor_program_sha256, "executor program")?;
    require_executable(&executor_program_path, "executor program")?;
    let executor_program_path = freeze_executor_program(
        &executor_program_path,
        &governance.paths.containers_root,
        &executor_program_sha256,
    )?;
    let evidence_root = ensure_current_attempt_layout(&governance, &identity, &request.attempt_id)?;
    let governance_path = archive_contract(
        &request.governance_path,
        &evidence_root.join("config-governance.yml"),
        "governance",
    )?;
    let pipeline_path = archive_contract(
        &request.pipeline_path,
        &evidence_root.join("config-pipeline.yml"),
        "pipeline",
    )?;
    let credential_reference_path = archive_contract(
        &request.credential_reference_path,
        &evidence_root.join("config-credential.yml"),
        "credential reference",
    )?;
    let historical_replay_source =
        archive_historical_source(historical_source_pipeline, &evidence_root)?;
    let governance_sha256 = bijux_dna_infra::hash_file_sha256(&governance_path)?;
    let pipeline_sha256 = bijux_dna_infra::hash_file_sha256(&pipeline_path)?;
    let credential_reference_sha256 =
        bijux_dna_infra::hash_file_sha256(&credential_reference_path)?;
    let spec = ContainerQualificationSlurmSpec {
        schema_version: CONTAINER_QUALIFICATION_SLURM_SPEC_SCHEMA_V5.to_string(),
        submission_kind,
        historical_replay_source,
        run_id: identity.run_id,
        tool_id: identity.tool_id,
        campaign_id: Some(identity.campaign_id),
        attempt_id: Some(request.attempt_id.clone()),
        job_name: identity.job_name,
        hostname: request.hostname.clone(),
        repository_path: identity.repository_path,
        repository_commit: identity.repository_commit,
        governance_path,
        governance_sha256,
        pipeline_path,
        pipeline_sha256,
        credential_reference_path,
        credential_reference_sha256,
        executor_program_path,
        executor_program_sha256,
        result_root: identity.result_root,
        evidence_root,
        resources: identity.resources,
        dependencies: identity.dependencies,
    };
    validate_specification(&spec)?;
    let specification_path = spec.evidence_root.join("qualification-spec.json");
    ensure_json_identity(&specification_path, &spec)?;
    let specification_sha256 = bijux_dna_infra::hash_file_sha256(&specification_path)?;

    let scheduler_root = ensure_directory(&spec.evidence_root.join("scheduler"))?;
    let script_path = scheduler_root.join("job.sbatch");
    let script = render_script(&spec, &specification_sha256)?;
    ensure_bytes_identity(&script_path, script.as_bytes())?;
    make_executable(&script_path)?;
    let script_sha256 = bijux_dna_infra::hash_file_sha256(&script_path)?;
    let prepared = PreparedContainerQualificationSlurmJob {
        schema_version: CONTAINER_QUALIFICATION_PREPARED_JOB_SCHEMA_V1.to_string(),
        request: SlurmSubmitRequest {
            run_id: format!("{}:{}", spec.run_id, spec.tool_id),
            dependencies: spec.dependencies.clone(),
            dependency_condition: SlurmDependencyCondition::AfterOk,
            array: None,
            script_path,
            working_directory: spec.repository_path.clone(),
            stdout_path: scheduler_root.join("stdout.log"),
            stderr_path: scheduler_root.join("stderr.log"),
            submitted_at_utc: submitted_at_utc.to_string(),
        },
        specification_sha256,
        script_sha256,
    };
    ensure_json_identity(&scheduler_root.join("prepared-job.json"), &prepared)?;
    Ok(prepared)
}

fn derive_preparation_identity(
    request: &ContainerQualificationPreparationRequest,
    historical_source_pipeline: Option<&Path>,
) -> Result<QualificationIdentity> {
    match historical_source_pipeline {
        Some(source) => derive_historical_replay_identity(
            &request.hostname,
            &request.governance_path,
            source,
            &request.pipeline_path,
            Some(&request.attempt_id),
            CONTAINER_QUALIFICATION_SLURM_SPEC_SCHEMA_V5,
        ),
        None => derive_identity(
            &request.hostname,
            &request.governance_path,
            &request.pipeline_path,
            Some(&request.attempt_id),
            CONTAINER_QUALIFICATION_SLURM_SPEC_SCHEMA_V5,
        ),
    }
}

fn archive_historical_source(
    source: Option<&Path>,
    evidence_root: &Path,
) -> Result<Option<HistoricalContainerQualificationReplaySource>> {
    source
        .map(|source| -> Result<HistoricalContainerQualificationReplaySource> {
            let pipeline_path = archive_contract(
                source,
                &evidence_root.join("source-config-pipeline.yml"),
                "historical source pipeline",
            )?;
            Ok(HistoricalContainerQualificationReplaySource {
                pipeline_sha256: bijux_dna_infra::hash_file_sha256(&pipeline_path)?,
                pipeline_path,
            })
        })
        .transpose()
}

fn render_script(
    spec: &ContainerQualificationSlurmSpec,
    specification_sha256: &str,
) -> Result<String> {
    let governance = load_container_governance(&spec.governance_path)?;
    let scratch_setup = if let Some(variable) = governance.slurm.scratch_environment {
        format!(
            "if [[ -z \"${{{variable}:-}}\" ]]; then\n  echo {} >&2\n  exit 72\nfi\nreadonly GOVERNED_SCRATCH_ROOT=\"${{{variable}}}\"",
            shell_quote(format!("{variable} is unavailable on the allocated node"))
        )
    } else {
        let root =
            governance.slurm.scratch_root.context("governance lacks a Slurm scratch authority")?;
        format!("readonly GOVERNED_SCRATCH_ROOT={}", shell_quote(root))
    };
    let required_scratch_kib = spec
        .resources
        .scratch_mb
        .checked_mul(1024)
        .context("qualification scratch requirement overflow")?;
    let command = qualification_command(spec);
    Ok([
        "#!/usr/bin/env bash".to_string(),
        format!("#SBATCH --job-name={}", spec.job_name),
        format!("#SBATCH --account={}", spec.resources.account),
        format!("#SBATCH --partition={}", spec.resources.partition),
        format!("#SBATCH --cpus-per-task={}", spec.resources.cpus),
        format!("#SBATCH --mem={}M", spec.resources.memory_mb),
        format!("#SBATCH --time={}", spec.resources.walltime),
        "#SBATCH --no-requeue".to_string(),
        String::new(),
        "set -euo pipefail".to_string(),
        "umask 077".to_string(),
        "export LANG=C".to_string(),
        "export LC_ALL=C".to_string(),
        "export TZ=UTC".to_string(),
        "[[ \"${SLURM_JOB_ID:-}\" =~ ^[0-9]+$ ]]".to_string(),
        format!("readonly REPOSITORY_ROOT={}", shell_quote(&spec.repository_path)),
        format!("readonly SPECIFICATION={}", shell_quote(spec.evidence_root.join("qualification-spec.json"))),
        format!("readonly EXECUTOR_PROGRAM={}", shell_quote(&spec.executor_program_path)),
        scratch_setup,
        "test -d \"${GOVERNED_SCRATCH_ROOT}\" && test ! -L \"${GOVERNED_SCRATCH_ROOT}\"".to_string(),
        "readonly JOB_SCRATCH=$(mktemp -d \"${GOVERNED_SCRATCH_ROOT%/}/bijux-genomics-${SLURM_JOB_ID}-XXXXXX\")".to_string(),
        "cleanup_scratch() { case \"${JOB_SCRATCH}\" in \"${GOVERNED_SCRATCH_ROOT%/}/bijux-genomics-${SLURM_JOB_ID}-\"*) rm -rf -- \"${JOB_SCRATCH}\" ;; *) return 73 ;; esac; }".to_string(),
        "trap cleanup_scratch EXIT".to_string(),
        "export TMPDIR=\"${JOB_SCRATCH}\"".to_string(),
        format!("test \"$(df -Pk \"${{TMPDIR}}\" | awk 'NR == 2 {{print $4}}')\" -ge {required_scratch_kib}"),
        "command -v git >/dev/null".to_string(),
        "command -v sha256sum >/dev/null".to_string(),
        "command -v awk >/dev/null".to_string(),
        format!("test \"$(git -C \"${{REPOSITORY_ROOT}}\" rev-parse HEAD)\" = {}", shell_quote(&spec.repository_commit)),
        "test -z \"$(git -C \"${REPOSITORY_ROOT}\" status --porcelain --untracked-files=all)\"".to_string(),
        hash_check("${SPECIFICATION}", specification_sha256),
        hash_check("${EXECUTOR_PROGRAM}", &spec.executor_program_sha256),
        hash_check_path(&spec.governance_path, &spec.governance_sha256),
        hash_check_path(&spec.pipeline_path, &spec.pipeline_sha256),
        hash_check_path(
            &spec.credential_reference_path,
            &spec.credential_reference_sha256,
        ),
        command,
        String::new(),
    ]
    .join("\n"))
}

fn qualification_command(spec: &ContainerQualificationSlurmSpec) -> String {
    [
        shell_quote(&spec.executor_program_path),
        "'containers'".to_string(),
        "'run'".to_string(),
        "'qualification-slurm'".to_string(),
        "'--'".to_string(),
        "'execute'".to_string(),
        shell_quote(spec.evidence_root.join("qualification-spec.json")),
    ]
    .join(" ")
}

fn hash_check(shell_path: &str, sha256: &str) -> String {
    format!("test \"$(sha256sum \"{shell_path}\" | awk '{{print $1}}')\" = {}", shell_quote(sha256))
}

fn hash_check_path(path: &Path, sha256: &str) -> String {
    format!(
        "test \"$(sha256sum {} | awk '{{print $1}}')\" = {}",
        shell_quote(path),
        shell_quote(sha256)
    )
}

fn ensure_current_attempt_layout(
    governance: &bijux_dna_containers::api::ContainerGovernance,
    identity: &QualificationIdentity,
    attempt_id: &str,
) -> Result<PathBuf> {
    let container_root = canonical_governed_directory(
        &governance.paths.containers_root,
        "governed containers root",
    )?
    .join("evidence/attempts");
    let benchmark_root =
        canonical_governed_directory(&governance.paths.results_root, "governed results root")?
            .join("benchmarks");
    bijux_dna_infra::validate_attempt_owner_roots(&container_root, &benchmark_root)
        .context("validate qualification attempt owner roots")?;
    let layout = bijux_dna_infra::ensure_attempt_layout(
        &container_root,
        bijux_dna_infra::AttemptEvidenceOwner::Containers,
        &bijux_dna_infra::AttemptLayoutIdentity {
            run_directory: identity.run_id.clone(),
            tool_id: identity.tool_id.clone(),
            campaign_id: identity.campaign_id.clone(),
            attempt_id: attempt_id.to_string(),
        },
    )
    .context("ensure governed qualification attempt layout")?;
    if layout.attempt_dir != identity.evidence_root {
        bail!("governed qualification attempt layout conflicts with admitted identity");
    }
    Ok(layout.attempt_dir)
}

fn ensure_directory(path: &Path) -> Result<PathBuf> {
    match fs::create_dir(path) {
        Ok(()) => {}
        Err(error) if error.kind() == ErrorKind::AlreadyExists => {}
        Err(error) => return Err(error).with_context(|| format!("create {}", path.display())),
    }
    canonical_directory(path, "qualification evidence directory")
}

fn archive_contract(source: &Path, target: &Path, label: &str) -> Result<PathBuf> {
    let metadata = fs::symlink_metadata(source).with_context(|| format!("inspect {label}"))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!("{label} source is not a regular file");
    }
    let bytes = bijux_dna_infra::read_to_end_bounded(source, 16 * 1024 * 1024)?;
    ensure_bytes_identity(target, &bytes)?;
    target.canonicalize().with_context(|| format!("canonicalize archived {label}"))
}

fn freeze_executor_program(source: &Path, containers_root: &Path, sha256: &str) -> Result<PathBuf> {
    let containers_root =
        canonical_governed_directory(containers_root, "governed containers root")?;
    let mut directory = containers_root.clone();
    for component in ["evidence", "executables", sha256] {
        directory = ensure_directory(&directory.join(component))?;
    }
    let target = directory.join("bijux-dna-dev");
    match fs::symlink_metadata(&target) {
        Ok(_) => {}
        Err(error) if error.kind() == ErrorKind::NotFound => {
            let mut input =
                fs::File::open(source).context("open qualification executor program")?;
            match bijux_dna_infra::atomic_create_with(&target, |output| {
                std::io::copy(&mut input, output)?;
                Ok(())
            }) {
                Ok(()) => {}
                Err(_error) if target.exists() => {
                    require_exact_file(&target, sha256, "frozen executor program")?;
                }
                Err(error) => return Err(error).context("freeze qualification executor program"),
            }
        }
        Err(error) => return Err(error).with_context(|| format!("inspect {}", target.display())),
    }
    let target = require_exact_file(&target, sha256, "frozen executor program")?;
    make_immutable_executable(&target)?;
    require_executable(&target, "frozen executor program")?;
    if !target.starts_with(&containers_root) {
        bail!("frozen executor program escapes governed containers root");
    }
    Ok(target)
}

fn ensure_bytes_identity(path: &Path, expected: &[u8]) -> Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                bail!("immutable qualification material is not a regular file");
            }
            if fs::read(path)? != expected {
                bail!("immutable qualification material conflicts at {}", path.display());
            }
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {
            bijux_dna_infra::atomic_create_bytes(path, expected)?;
        }
        Err(error) => return Err(error).with_context(|| format!("inspect {}", path.display())),
    }
    Ok(())
}

fn make_executable(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        let mut permissions = fs::metadata(path)?.permissions();
        permissions.set_mode(0o750);
        fs::set_permissions(path, permissions)?;
    }
    Ok(())
}

fn make_immutable_executable(path: &Path) -> Result<()> {
    let mut permissions = fs::metadata(path)?.permissions();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        permissions.set_mode(0o550);
    }
    #[cfg(not(unix))]
    permissions.set_readonly(true);
    fs::set_permissions(path, permissions)?;
    Ok(())
}

fn shell_quote(value: impl AsRef<OsStr>) -> String {
    let value = value.as_ref().to_string_lossy();
    format!("'{}'", value.replace('\'', "'\\''"))
}
