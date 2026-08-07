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
    PreparedContainerQualificationSlurmJob, CONTAINER_QUALIFICATION_PREPARED_JOB_SCHEMA_V1,
    CONTAINER_QUALIFICATION_SLURM_SPEC_SCHEMA_V1,
};
use super::validation::{
    canonical_directory, derive_identity, ensure_json_identity, require_exact_file,
    require_utc_timestamp, validate_specification,
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
    require_utc_timestamp(submitted_at_utc, "submission timestamp")?;
    load_container_credential_reference(&request.credential_reference_path)?;
    let identity =
        derive_identity(&request.hostname, &request.governance_path, &request.pipeline_path)?;
    let evidence_root = ensure_evidence_root(&identity.evidence_root)?;
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
    let governance_sha256 = bijux_dna_infra::hash_file_sha256(&governance_path)?;
    let pipeline_sha256 = bijux_dna_infra::hash_file_sha256(&pipeline_path)?;
    let credential_reference_sha256 =
        bijux_dna_infra::hash_file_sha256(&credential_reference_path)?;
    let executor_program_path = request
        .executor_program_path
        .canonicalize()
        .context("canonicalize qualification executor program")?;
    let executor_program_sha256 = bijux_dna_infra::hash_file_sha256(&executor_program_path)?;
    require_exact_file(&executor_program_path, &executor_program_sha256, "executor program")?;
    let executor_script_path =
        identity.repository_path.join("containers/contracts/container_validation.py");
    let executor_script_sha256 = bijux_dna_infra::hash_file_sha256(&executor_script_path)?;
    require_exact_file(&executor_script_path, &executor_script_sha256, "executor script")?;

    let spec = ContainerQualificationSlurmSpec {
        schema_version: CONTAINER_QUALIFICATION_SLURM_SPEC_SCHEMA_V1.to_string(),
        run_id: identity.run_id,
        tool_id: identity.tool_id,
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
        executor_script_path,
        executor_script_sha256,
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
    let command = [
        shell_quote(&spec.executor_program_path),
        shell_quote(&spec.executor_script_path),
        "'execute'".to_string(),
        "'--governance'".to_string(),
        shell_quote(&spec.governance_path),
        "'--pipeline'".to_string(),
        shell_quote(&spec.pipeline_path),
        "'--credentials'".to_string(),
        shell_quote(&spec.credential_reference_path),
    ]
    .join(" ");
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
        format!("readonly EXECUTOR_SCRIPT={}", shell_quote(&spec.executor_script_path)),
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
        hash_check("${EXECUTOR_SCRIPT}", &spec.executor_script_sha256),
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

fn ensure_evidence_root(path: &Path) -> Result<PathBuf> {
    let parent = path.parent().context("qualification evidence root has no parent")?;
    fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    ensure_directory(path)
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

fn shell_quote(value: impl AsRef<OsStr>) -> String {
    let value = value.as_ref().to_string_lossy();
    format!("'{}'", value.replace('\'', "'\\''"))
}
