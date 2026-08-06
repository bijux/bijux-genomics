//! Owner: bijux-dna-runner
//! Identity and catalog closure checks for corpus materialization jobs.

use std::fs;
use std::path::{Component, Path, PathBuf};

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{
    validate_corpus_acquisition_plan, validate_governed_data_catalog, CorpusAcquisitionPlan,
    GovernedDataCatalog,
};
use serde::de::DeserializeOwned;

use super::contracts::{
    CorpusMaterializationSlurmSpec, CORPUS_MATERIALIZATION_SLURM_SPEC_SCHEMA_V1,
};

pub(super) const MAX_EVIDENCE_BYTES: usize = 16 * 1024 * 1024;

pub(super) fn validate_specification(
    spec: &CorpusMaterializationSlurmSpec,
) -> Result<CorpusAcquisitionPlan> {
    if spec.schema_version != CORPUS_MATERIALIZATION_SLURM_SPEC_SCHEMA_V1 {
        bail!("corpus materialization Slurm specification schema is unsupported");
    }
    require_run_id(&spec.run_id)?;
    require_identifier(&spec.attempt_id, "attempt")?;
    require_identifier(&spec.portfolio_id, "portfolio")?;
    require_git_commit(&spec.repository_commit)?;
    require_sha256(&spec.executable_sha256, "executable SHA-256")?;
    require_sha256(&spec.plan_sha256, "plan SHA-256")?;
    require_scheduler_token(&spec.resources.account, "account")?;
    require_scheduler_token(&spec.resources.partition, "partition")?;
    require_walltime(&spec.resources.walltime)?;
    if spec.resources.cpus == 0 || spec.resources.cpus > 64 {
        bail!("corpus materialization CPUs must be between 1 and 64");
    }
    if spec.resources.memory_mb < 256 || spec.resources.memory_mb > 1_048_576 {
        bail!("corpus materialization memory must be between 256 and 1048576 MiB");
    }
    if spec.maximum_parallelism == 0
        || spec.maximum_parallelism > 64
        || usize::try_from(spec.resources.cpus).ok() != Some(spec.maximum_parallelism)
    {
        bail!(
            "corpus materialization parallelism must equal requested CPUs and be between 1 and 64"
        );
    }
    if spec.retries > 10 || spec.timeout_seconds == 0 || spec.timeout_seconds > 3600 {
        bail!("corpus materialization retry or timeout policy is invalid");
    }
    require_utc_timestamp(&spec.admitted_at_utc, "admission timestamp")?;

    let data_root = canonical_directory(&spec.data_root, "governed data root")?;
    canonical_directory(&spec.repository_path, "frozen repository")?;
    require_exact_file(
        &spec.executable_path,
        &spec.executable_sha256,
        "materialization executable",
    )?;
    let plan_path = require_exact_file(&spec.plan_path, &spec.plan_sha256, "acquisition plan")?;
    plan_path.strip_prefix(&data_root).context("acquisition plan escapes governed data root")?;
    let plan: CorpusAcquisitionPlan = read_json(&plan_path)?;
    validate_corpus_acquisition_plan(&plan).context("validate corpus acquisition plan")?;
    if plan.portfolio_id != spec.portfolio_id {
        bail!("acquisition plan portfolio differs from scheduler specification");
    }
    if !spec.evidence_root.is_absolute()
        || spec
            .evidence_root
            .components()
            .any(|component| matches!(component, Component::ParentDir | Component::CurDir))
    {
        bail!("corpus materialization evidence root must be an absolute normalized path");
    }
    let materialization_evidence_root = data_root.join("evidence/materializations");
    let relative_evidence_root = spec
        .evidence_root
        .strip_prefix(&materialization_evidence_root)
        .context("corpus materialization evidence escapes governed materialization root")?;
    if relative_evidence_root.as_os_str().is_empty() {
        bail!("corpus materialization evidence root must identify one immutable run");
    }
    Ok(plan)
}

pub(super) fn validate_catalog_closure(
    spec: &CorpusMaterializationSlurmSpec,
    plan: &CorpusAcquisitionPlan,
) -> Result<(PathBuf, String)> {
    let catalog_path = spec.data_root.join("catalog").join(format!("{}.json", spec.portfolio_id));
    let catalog: GovernedDataCatalog = read_json(&catalog_path)?;
    validate_governed_data_catalog(&catalog).context("validate governed data catalog")?;
    let expected_file_count =
        usize::try_from(plan.file_count).context("acquisition plan file count exceeds platform")?;
    if catalog.id != spec.portfolio_id || catalog.files.len() != expected_file_count {
        bail!("governed catalog identity or file count differs from acquisition plan");
    }
    let corpus_root = spec.data_root.join("corpora").join(&spec.portfolio_id).join("raw");
    let mut observed_size = 0_u64;
    for file in &catalog.files {
        let path = Path::new(&file.path);
        let metadata = fs::symlink_metadata(path)
            .with_context(|| format!("inspect admitted FASTQ {}", path.display()))?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            bail!("admitted FASTQ is not a regular non-symlink file: {}", path.display());
        }
        let canonical = path
            .canonicalize()
            .with_context(|| format!("canonicalize admitted FASTQ {}", path.display()))?;
        canonical
            .strip_prefix(&corpus_root)
            .context("admitted FASTQ escapes governed corpus root")?;
        if canonical != path || metadata.len() != file.size_bytes {
            bail!("admitted FASTQ path or size differs from catalog: {}", path.display());
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            if metadata.permissions().mode() & 0o222 != 0 {
                bail!("admitted FASTQ remains writable: {}", path.display());
            }
        }
        let sha256 = bijux_dna_infra::hash_file_sha256(path)
            .with_context(|| format!("hash admitted FASTQ {}", path.display()))?;
        if sha256 != file.sha256 {
            bail!("admitted FASTQ digest differs from catalog: {}", path.display());
        }
        observed_size = observed_size
            .checked_add(file.size_bytes)
            .context("admitted FASTQ byte total overflow")?;
    }
    if observed_size != plan.total_size_bytes {
        bail!("governed catalog byte total differs from acquisition plan");
    }
    let catalog_sha256 =
        bijux_dna_infra::hash_file_sha256(&catalog_path).context("hash governed data catalog")?;
    Ok((catalog_path, catalog_sha256))
}

pub(super) fn require_utc_timestamp(value: &str, label: &str) -> Result<()> {
    bijux_dna_bench_model::validate_utc_timestamp(value, label).map_err(Into::into)
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
            let existing: T = read_json(path)?;
            if existing != *value {
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

pub(super) fn require_exact_file(path: &Path, sha256: &str, label: &str) -> Result<PathBuf> {
    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("inspect {label} {}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!("{label} is not a regular non-symlink file");
    }
    #[cfg(unix)]
    if label == "materialization executable" {
        use std::os::unix::fs::PermissionsExt as _;
        if metadata.permissions().mode() & 0o111 == 0 {
            bail!("materialization executable has no executable permission");
        }
    }
    let canonical = path.canonicalize().with_context(|| format!("canonicalize {label}"))?;
    if canonical != path {
        bail!("{label} path is not canonical");
    }
    let observed =
        bijux_dna_infra::hash_file_sha256(path).with_context(|| format!("hash {label}"))?;
    if observed != sha256 {
        bail!("{label} digest differs from scheduler specification");
    }
    Ok(canonical)
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

fn require_run_id(value: &str) -> Result<()> {
    if value.len() != 8
        || !value.starts_with("run-")
        || !value[4..].bytes().all(|byte| byte.is_ascii_digit())
    {
        bail!("corpus materialization run id must match run-<four digits>");
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
        bail!("corpus materialization {label} identity is invalid");
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
