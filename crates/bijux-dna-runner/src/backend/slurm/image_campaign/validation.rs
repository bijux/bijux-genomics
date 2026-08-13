//! Owner: bijux-dna-runner
//! Image campaign scheduler authority validation.

use std::fs;
use std::path::{Component, Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde::de::DeserializeOwned;

use super::contracts::{ImageCampaignSlurmSpec, IMAGE_CAMPAIGN_SLURM_SPEC_SCHEMA_V1};

const MAX_EVIDENCE_BYTES: usize = 16 * 1024 * 1024;

/// Load and validate one immutable image campaign scheduler specification.
///
/// # Errors
/// Returns an error for malformed JSON, unsupported schema, authority drift, an invalid tool
/// closure, unsafe resources, or a path outside the governed run layout.
pub fn load_image_campaign_slurm_spec(path: &Path) -> Result<ImageCampaignSlurmSpec> {
    let spec = read_json(path)?;
    validate_specification(&spec)?;
    Ok(spec)
}

pub(super) fn validate_specification(spec: &ImageCampaignSlurmSpec) -> Result<()> {
    if spec.schema_version != IMAGE_CAMPAIGN_SLURM_SPEC_SCHEMA_V1 {
        bail!("unsupported image campaign Slurm specification schema");
    }
    if spec.run_id != "run-1000" {
        bail!("image campaign must use permanent run-1000 protocol");
    }
    require_identifier(&spec.attempt_id, "attempt")?;
    require_git_commit(&spec.repository_commit)?;
    require_sha256(&spec.executable_sha256, "executable SHA-256")?;
    require_sha256(&spec.apptainer_sha256, "Apptainer SHA-256")?;
    require_scheduler_token(&spec.resources.account, "account")?;
    require_scheduler_token(&spec.resources.partition, "partition")?;
    require_walltime(&spec.resources.walltime)?;
    if spec.resources.cpus == 0 || spec.resources.cpus > 32 {
        bail!("image campaign CPUs must be between 1 and 32");
    }
    if spec.resources.memory_mb < 1024 || spec.resources.memory_mb > 1_048_576 {
        bail!("image campaign memory must be between 1024 and 1048576 MiB");
    }
    if spec.worker_count == 0
        || spec.worker_count > 32
        || usize::try_from(spec.resources.cpus).ok() != Some(spec.worker_count)
    {
        bail!("image campaign worker count must equal requested CPUs and be between 1 and 32");
    }
    let repository = canonical_directory(&spec.repository_path, "image campaign repository")?;
    if !bijux_dna_containers::api::repository_commit_is_ancestor_of_head(
        &repository,
        &spec.repository_commit,
    )? {
        bail!("image campaign repository commit is not retained in source history");
    }
    let study_root = canonical_directory(&spec.study_root, "governed STUDY root")?;
    validate_attempt_root(spec, &study_root)?;
    validate_tool_identifiers(&spec.selected_tools)?;
    Ok(())
}

pub(super) fn validate_submission_authorities(spec: &ImageCampaignSlurmSpec) -> Result<()> {
    let repository = canonical_directory(&spec.repository_path, "image campaign repository")?;
    if bijux_dna_containers::api::clean_repository_head_commit(&repository)?
        != spec.repository_commit
    {
        bail!("image campaign repository commit differs from scheduler specification");
    }
    require_exact_file(
        &spec.executable_path,
        &spec.executable_sha256,
        "campaign executable",
        true,
    )?;
    require_exact_file(&spec.apptainer_path, &spec.apptainer_sha256, "Apptainer executable", true)?;
    validate_tool_selection(spec, &repository)?;
    Ok(())
}

fn validate_attempt_root(spec: &ImageCampaignSlurmSpec, study_root: &Path) -> Result<()> {
    if !spec.attempt_root.is_absolute()
        || spec
            .attempt_root
            .components()
            .any(|component| matches!(component, Component::ParentDir | Component::CurDir))
    {
        bail!("image campaign attempt root must be an absolute normalized path");
    }
    let expected = study_root
        .join("containers/evidence/image-campaign-submissions")
        .join(&spec.repository_commit)
        .join("attempts")
        .join(&spec.attempt_id);
    if spec.attempt_root != expected {
        bail!("image campaign attempt root differs from governed run layout");
    }
    match fs::symlink_metadata(&spec.attempt_root) {
        Ok(_) => {
            canonical_directory(&spec.attempt_root, "image campaign attempt root")?;
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(error).with_context(|| format!("inspect {}", spec.attempt_root.display()));
        }
    }
    Ok(())
}

fn validate_tool_selection(spec: &ImageCampaignSlurmSpec, repository: &Path) -> Result<()> {
    if spec.selected_tools.is_empty() {
        bail!("image campaign selected tool closure is empty");
    }
    let catalog = bijux_dna_containers::api::load_container_catalog(repository)?;
    let retained = bijux_dna_containers::api::retained_apptainer_tool_ids(
        repository,
        &catalog.apptainer_architecture,
    )?;
    let selected = retained
        .iter()
        .filter(|tool| spec.selected_tools.contains(tool))
        .cloned()
        .collect::<Vec<_>>();
    if selected != spec.selected_tools {
        bail!("image campaign tools must be unique retained tools in canonical catalog order");
    }
    Ok(())
}

fn validate_tool_identifiers(tools: &[String]) -> Result<()> {
    if tools.is_empty() {
        bail!("image campaign selected tool closure is empty");
    }
    let mut observed = std::collections::BTreeSet::new();
    for tool in tools {
        if tool.is_empty()
            || tool.len() > 96
            || !tool.bytes().all(|byte| {
                byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_')
            })
        {
            bail!("image campaign tool identity is invalid");
        }
        if !observed.insert(tool) {
            bail!("image campaign selected tools are duplicated");
        }
    }
    Ok(())
}

pub(super) fn read_json<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let metadata =
        fs::symlink_metadata(path).with_context(|| format!("inspect {}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!("JSON authority is not a regular non-symlink file: {}", path.display());
    }
    let bytes = bijux_dna_infra::read_to_end_bounded(path, MAX_EVIDENCE_BYTES)
        .with_context(|| format!("read {}", path.display()))?;
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
            bijux_dna_infra::atomic_create_json(path, value)
                .with_context(|| format!("publish immutable JSON evidence {}", path.display()))?;
        }
        Err(error) => return Err(error).with_context(|| format!("inspect {}", path.display())),
    }
    Ok(())
}

pub(super) fn canonical_directory(path: &Path, label: &str) -> Result<PathBuf> {
    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("inspect {label} {}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        bail!("{label} is not a regular non-symlink directory");
    }
    let canonical = path.canonicalize().with_context(|| format!("canonicalize {label}"))?;
    if canonical != path {
        bail!("{label} path is not canonical");
    }
    Ok(canonical)
}

pub(super) fn require_exact_file(
    path: &Path,
    sha256: &str,
    label: &str,
    executable: bool,
) -> Result<PathBuf> {
    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("inspect {label} {}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!("{label} is not a regular non-symlink file");
    }
    #[cfg(unix)]
    if executable {
        use std::os::unix::fs::PermissionsExt as _;
        if metadata.permissions().mode() & 0o111 == 0 {
            bail!("{label} has no executable permission");
        }
    }
    let canonical = path.canonicalize().with_context(|| format!("canonicalize {label}"))?;
    if canonical != path {
        bail!("{label} path is not canonical");
    }
    if bijux_dna_infra::hash_file_sha256(path)? != sha256 {
        bail!("{label} digest differs from scheduler specification");
    }
    Ok(canonical)
}

fn require_identifier(value: &str, label: &str) -> Result<()> {
    if value.is_empty()
        || value.len() > 96
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        bail!("image campaign {label} identity is invalid");
    }
    Ok(())
}

fn require_sha256(value: &str, label: &str) -> Result<()> {
    if value.len() != 64
        || !value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        bail!("{label} must be a lowercase SHA-256 digest");
    }
    Ok(())
}

fn require_git_commit(value: &str) -> Result<()> {
    if value.len() != 40
        || !value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        bail!("repository commit must be a lowercase 40-character Git object id");
    }
    Ok(())
}

fn require_scheduler_token(value: &str, label: &str) -> Result<()> {
    if value.is_empty()
        || value.len() > 96
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':'))
    {
        bail!("Slurm {label} contains unsupported characters");
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
        bail!("Slurm walltime must use HH:MM:SS");
    }
    let hours = parts[0].parse::<u64>().context("parse Slurm walltime hours")?;
    let minutes = parts[1].parse::<u64>().context("parse Slurm walltime minutes")?;
    let seconds = parts[2].parse::<u64>().context("parse Slurm walltime seconds")?;
    if hours > 999 || minutes > 59 || seconds > 59 || (hours == 0 && minutes == 0 && seconds == 0) {
        bail!("Slurm walltime is outside supported bounds");
    }
    Ok(())
}
