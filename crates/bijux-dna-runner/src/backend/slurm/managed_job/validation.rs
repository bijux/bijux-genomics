//! Validation and bounded JSON loading for immutable worker authorities.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde::de::DeserializeOwned;

use super::super::SlurmDependencyCondition;
use super::contracts::{ManagedSlurmJobSpec, MANAGED_SLURM_JOB_SPEC_SCHEMA_V1};

const MAX_EVIDENCE_BYTES: usize = 16 * 1024 * 1024;

/// Load and validate one immutable managed worker specification.
///
/// # Errors
/// Returns an error for malformed JSON, unsafe paths or resources, or identity drift.
pub fn load_managed_slurm_job_spec(path: &Path) -> Result<ManagedSlurmJobSpec> {
    let spec: ManagedSlurmJobSpec = read_json(path)?;
    validate_spec(&spec)?;
    let expected = spec.attempt_root.join("managed-slurm-job-spec.json");
    if path.canonicalize().context("canonicalize managed Slurm specification")? != expected {
        bail!("managed Slurm specification must reside at its attempt root");
    }
    Ok(spec)
}

pub(super) fn validate_spec(spec: &ManagedSlurmJobSpec) -> Result<()> {
    if spec.schema_version != MANAGED_SLURM_JOB_SPEC_SCHEMA_V1 {
        bail!("unsupported managed Slurm job specification schema");
    }
    require_identifier(&spec.run_id, "run")?;
    require_identifier(&spec.attempt_id, "attempt")?;
    require_git_commit(&spec.repository_commit)?;
    require_sha256(&spec.worker_sha256, "worker SHA-256")?;
    require_scheduler_token(&spec.resources.account, "account")?;
    require_scheduler_token(&spec.resources.partition, "partition")?;
    require_walltime(&spec.resources.walltime)?;
    validate_timestamp(&spec.submitted_at_utc)?;
    if spec.resources.cpus == 0 || spec.resources.cpus > 256 {
        bail!("managed Slurm CPUs must be between 1 and 256");
    }
    if spec.resources.memory_mb < 128 || spec.resources.memory_mb > 1_048_576 {
        bail!("managed Slurm memory must be between 128 and 1048576 MiB");
    }
    let repository = canonical_directory(&spec.repository_path, "repository")?;
    if !bijux_dna_containers::api::repository_commit_is_ancestor_of_head(
        &repository,
        &spec.repository_commit,
    )? {
        bail!("managed Slurm repository commit is not retained in source history");
    }
    require_exact_file(&spec.worker_path, &spec.worker_sha256, "worker")?;
    let attempt_root = canonical_directory(&spec.attempt_root, "attempt root")?;
    if attempt_root.starts_with(&repository) {
        bail!("managed Slurm attempt root must be outside the immutable repository");
    }
    let dependencies = spec.dependencies.iter().collect::<std::collections::BTreeSet<_>>();
    if dependencies.len() != spec.dependencies.len()
        || spec
            .dependencies
            .iter()
            .any(|id| id.is_empty() || !id.bytes().all(|byte| byte.is_ascii_digit()))
    {
        bail!("managed Slurm dependency identity is malformed");
    }
    if spec.dependencies.is_empty()
        && spec.dependency_condition != SlurmDependencyCondition::AfterOk
    {
        bail!("managed Slurm afterany requires at least one dependency");
    }
    Ok(())
}

pub(super) fn validate_submission_source(spec: &ManagedSlurmJobSpec) -> Result<()> {
    let repository = canonical_directory(&spec.repository_path, "repository")?;
    if bijux_dna_containers::api::clean_repository_head_commit(&repository)?
        != spec.repository_commit
    {
        bail!("managed Slurm repository commit differs from specification");
    }
    Ok(())
}

pub(super) fn read_json<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let metadata =
        fs::symlink_metadata(path).with_context(|| format!("inspect {}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!("managed Slurm JSON authority is not a regular file");
    }
    let bytes = bijux_dna_infra::read_to_end_bounded(path, MAX_EVIDENCE_BYTES)?;
    serde_json::from_slice(&bytes).with_context(|| format!("parse {}", path.display()))
}

pub(super) fn canonical_directory(path: &Path, label: &str) -> Result<PathBuf> {
    let metadata = fs::symlink_metadata(path).with_context(|| format!("inspect {label}"))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        bail!("managed Slurm {label} is not a regular directory");
    }
    let canonical = path.canonicalize().with_context(|| format!("canonicalize {label}"))?;
    if canonical != path {
        bail!("managed Slurm {label} path is not canonical");
    }
    Ok(canonical)
}

pub(super) fn validate_timestamp(value: &str) -> Result<()> {
    let parsed = chrono::DateTime::parse_from_rfc3339(value)
        .context("managed Slurm timestamp is invalid")?;
    if parsed.offset().local_minus_utc() != 0 || !value.ends_with('Z') {
        bail!("managed Slurm timestamp must use UTC Z notation");
    }
    Ok(())
}

fn require_exact_file(path: &Path, sha256: &str, label: &str) -> Result<()> {
    let metadata = fs::symlink_metadata(path).with_context(|| format!("inspect {label}"))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!("managed Slurm {label} is not a regular file");
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        if metadata.permissions().mode() & 0o111 == 0 {
            bail!("managed Slurm {label} is not executable");
        }
    }
    if path.canonicalize()? != path || bijux_dna_infra::hash_file_sha256(path)? != sha256 {
        bail!("managed Slurm {label} identity differs from specification");
    }
    Ok(())
}

fn require_identifier(value: &str, label: &str) -> Result<()> {
    if value.is_empty()
        || value.len() > 96
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        bail!("managed Slurm {label} identity is invalid");
    }
    Ok(())
}

fn require_git_commit(value: &str) -> Result<()> {
    if value.len() != 40 || !value.bytes().all(is_lower_hex) {
        bail!("managed Slurm repository commit is invalid");
    }
    Ok(())
}

fn require_sha256(value: &str, label: &str) -> Result<()> {
    if value.len() != 64 || !value.bytes().all(is_lower_hex) {
        bail!("managed Slurm {label} is invalid");
    }
    Ok(())
}

fn is_lower_hex(byte: u8) -> bool {
    byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)
}

fn require_scheduler_token(value: &str, label: &str) -> Result<()> {
    if value.is_empty()
        || value.len() > 96
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':'))
    {
        bail!("managed Slurm {label} is invalid");
    }
    Ok(())
}

fn require_walltime(value: &str) -> Result<()> {
    let parts = value.split(':').collect::<Vec<_>>();
    if parts.len() != 3
        || parts
            .iter()
            .any(|part| part.is_empty() || !part.bytes().all(|byte| byte.is_ascii_digit()))
    {
        bail!("managed Slurm walltime must use HH:MM:SS");
    }
    let hours = parts[0].parse::<u64>()?;
    let minutes = parts[1].parse::<u64>()?;
    let seconds = parts[2].parse::<u64>()?;
    if hours > 999 || minutes > 59 || seconds > 59 || hours + minutes + seconds == 0 {
        bail!("managed Slurm walltime is outside supported bounds");
    }
    Ok(())
}
