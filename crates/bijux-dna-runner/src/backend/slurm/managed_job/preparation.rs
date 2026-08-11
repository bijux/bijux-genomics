//! Immutable scheduler material preparation for exact external workers.

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde::{de::DeserializeOwned, Serialize};

use super::super::SlurmSubmitRequest;
use super::contracts::{
    ManagedSlurmJobSpec, PreparedManagedSlurmJob, PREPARED_MANAGED_SLURM_JOB_SCHEMA_V1,
};
use super::validation::{
    canonical_directory, read_json, validate_spec, validate_submission_source,
};

/// Prepare immutable scheduler material without submitting or waiting.
///
/// # Errors
/// Returns an error for invalid authority, conflicting retained material, or unsafe scheduler data.
pub fn prepare_managed_slurm_job(spec: &ManagedSlurmJobSpec) -> Result<PreparedManagedSlurmJob> {
    validate_spec(spec)?;
    validate_submission_source(spec)?;
    let specification_path = spec.attempt_root.join("managed-slurm-job-spec.json");
    ensure_json_identity(&specification_path, spec)?;
    let specification_sha256 = bijux_dna_infra::hash_file_sha256(&specification_path)?;
    let scheduler_root = ensure_directory(&spec.attempt_root.join("scheduler"))?;
    let script_path = scheduler_root.join("job.sbatch");
    let script = render_script(spec, &specification_sha256);
    ensure_bytes_identity(&script_path, script.as_bytes())?;
    make_executable(&script_path)?;
    let script_sha256 = bijux_dna_infra::hash_file_sha256(&script_path)?;
    let prepared = PreparedManagedSlurmJob {
        schema_version: PREPARED_MANAGED_SLURM_JOB_SCHEMA_V1.to_string(),
        request: SlurmSubmitRequest {
            run_id: format!("{}:{}", spec.run_id, spec.attempt_id),
            dependencies: spec.dependencies.clone(),
            dependency_condition: spec.dependency_condition,
            array: None,
            script_path,
            working_directory: spec.repository_path.clone(),
            stdout_path: scheduler_root.join("stdout.log"),
            stderr_path: scheduler_root.join("stderr.log"),
            submitted_at_utc: spec.submitted_at_utc.clone(),
        },
        specification_sha256,
        worker_sha256: spec.worker_sha256.clone(),
        script_sha256,
    };
    ensure_json_identity(&scheduler_root.join("prepared-job.json"), &prepared)?;
    Ok(prepared)
}

pub(super) fn ensure_json_identity<T>(path: &Path, value: &T) -> Result<()>
where
    T: Serialize + DeserializeOwned + PartialEq,
{
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                bail!("managed Slurm JSON evidence is not a regular file");
            }
            if read_json::<T>(path)? != *value {
                bail!("managed Slurm JSON evidence conflicts with immutable content");
            }
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {
            bijux_dna_infra::atomic_create_json(path, value)?;
        }
        Err(error) => return Err(error).with_context(|| format!("inspect {}", path.display())),
    }
    Ok(())
}

fn render_script(spec: &ManagedSlurmJobSpec, specification_sha256: &str) -> String {
    [
        "#!/usr/bin/env bash".to_string(),
        format!("#SBATCH --job-name=bijux-{}", spec.attempt_id),
        format!("#SBATCH --account={}", spec.resources.account),
        format!("#SBATCH --partition={}", spec.resources.partition),
        format!("#SBATCH --cpus-per-task={}", spec.resources.cpus),
        format!("#SBATCH --mem={}M", spec.resources.memory_mb),
        format!("#SBATCH --time={}", spec.resources.walltime),
        String::new(),
        "set -euo pipefail".to_string(),
        "umask 077".to_string(),
        format!("readonly REPOSITORY_ROOT={}", shell_quote(&spec.repository_path)),
        format!("readonly WORKER={}", shell_quote(&spec.worker_path)),
        format!("readonly ATTEMPT_ROOT={}", shell_quote(&spec.attempt_root)),
        format!(
            "readonly SPECIFICATION={}",
            shell_quote(spec.attempt_root.join("managed-slurm-job-spec.json"))
        ),
        "mkdir \"${ATTEMPT_ROOT}/execution-lock\"".to_string(),
        format!(
            "test \"$(git -C \"${{REPOSITORY_ROOT}}\" rev-parse HEAD)\" = {}",
            shell_quote(&spec.repository_commit)
        ),
        "test -z \"$(git -C \"${REPOSITORY_ROOT}\" status --porcelain --untracked-files=all)\""
            .to_string(),
        format!(
            "test \"$(sha256sum \"${{WORKER}}\" | awk '{{print $1}}')\" = {}",
            shell_quote(&spec.worker_sha256)
        ),
        format!(
            "test \"$(sha256sum \"${{SPECIFICATION}}\" | awk '{{print $1}}')\" = {}",
            shell_quote(specification_sha256)
        ),
        "exec \"${WORKER}\"".to_string(),
        String::new(),
    ]
    .join("\n")
}

fn ensure_directory(path: &Path) -> Result<PathBuf> {
    match fs::create_dir(path) {
        Ok(()) => {}
        Err(error) if error.kind() == ErrorKind::AlreadyExists => {}
        Err(error) => return Err(error).with_context(|| format!("create {}", path.display())),
    }
    canonical_directory(path, "scheduler directory")
}

fn ensure_bytes_identity(path: &Path, expected: &[u8]) -> Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink()
                || !metadata.is_file()
                || fs::read(path)? != expected
            {
                bail!("managed Slurm script conflicts with immutable specification");
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

fn shell_quote(value: impl AsRef<std::ffi::OsStr>) -> String {
    let value = value.as_ref().to_string_lossy();
    format!("'{}'", value.replace('\'', "'\\''"))
}
