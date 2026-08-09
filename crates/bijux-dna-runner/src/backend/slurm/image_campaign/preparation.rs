//! Owner: bijux-dna-runner
//! Immutable scheduler material for container image campaigns.

use std::ffi::OsStr;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

use super::contracts::{
    ImageCampaignSlurmSpec, PreparedImageCampaignSlurmJob, IMAGE_CAMPAIGN_PREPARED_JOB_SCHEMA_V1,
};
use super::validation::{canonical_directory, ensure_json_identity, validate_specification};
use crate::backend::slurm::{SlurmDependencyCondition, SlurmSubmitRequest};

/// Validate campaign authorities and publish restart-safe scheduler material without submitting.
///
/// # Errors
/// Returns an error for source drift, invalid run layout, conflicting evidence, unsafe resources,
/// or an invalid submission timestamp.
pub fn prepare_image_campaign_slurm_job(
    spec: &ImageCampaignSlurmSpec,
    submitted_at_utc: &str,
) -> Result<PreparedImageCampaignSlurmJob> {
    validate_specification(spec)?;
    require_utc_timestamp(submitted_at_utc)?;
    let attempt_root = ensure_attempt_root(spec)?;
    let specification_path = attempt_root.join("image-campaign-spec.json");
    ensure_json_identity(&specification_path, spec)?;
    let specification_sha256 = bijux_dna_infra::hash_file_sha256(&specification_path)
        .context("hash image campaign Slurm specification")?;
    let scheduler_root = ensure_directory(&attempt_root.join("scheduler"))?;
    let script_path = scheduler_root.join("job.sbatch");
    ensure_bytes_identity(&script_path, render_script(spec, &specification_sha256).as_bytes())?;
    make_executable(&script_path)?;
    let script_sha256 =
        bijux_dna_infra::hash_file_sha256(&script_path).context("hash image campaign script")?;
    let prepared = PreparedImageCampaignSlurmJob {
        schema_version: IMAGE_CAMPAIGN_PREPARED_JOB_SCHEMA_V1.to_string(),
        request: SlurmSubmitRequest {
            run_id: format!("{}:{}", spec.run_id, spec.attempt_id),
            dependencies: Vec::new(),
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

fn render_script(spec: &ImageCampaignSlurmSpec, specification_sha256: &str) -> String {
    let tools = spec.selected_tools.join(",");
    let command = [
        shell_quote(&spec.executable_path),
        "'containers'".to_string(),
        "'run'".to_string(),
        "'image-campaign'".to_string(),
        "'--'".to_string(),
        "'execute'".to_string(),
        shell_quote(&spec.repository_path),
        shell_quote(&spec.study_root),
        shell_quote(&spec.apptainer_path),
        shell_quote(&spec.attempt_root),
        shell_quote(&spec.attempt_id),
        shell_quote(spec.worker_count.to_string()),
        shell_quote(tools),
        "\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"".to_string(),
    ]
    .join(" ");
    [
        "#!/usr/bin/env bash".to_string(),
        format!("#SBATCH --job-name=bijux-images-{}", spec.attempt_id),
        format!("#SBATCH --account={}", spec.resources.account),
        format!("#SBATCH --partition={}", spec.resources.partition),
        format!("#SBATCH --cpus-per-task={}", spec.resources.cpus),
        format!("#SBATCH --mem={}M", spec.resources.memory_mb),
        format!("#SBATCH --time={}", spec.resources.walltime),
        String::new(),
        "set -euo pipefail".to_string(),
        "umask 077".to_string(),
        format!("readonly REPOSITORY_ROOT={}", shell_quote(&spec.repository_path)),
        format!("readonly EXECUTABLE={}", shell_quote(&spec.executable_path)),
        format!("readonly APPTAINER={}", shell_quote(&spec.apptainer_path)),
        format!("readonly ATTEMPT_ROOT={}", shell_quote(&spec.attempt_root)),
        format!(
            "readonly SPECIFICATION={}",
            shell_quote(spec.attempt_root.join("image-campaign-spec.json"))
        ),
        "mkdir \"${ATTEMPT_ROOT}/execution-lock\"".to_string(),
        "command -v git >/dev/null".to_string(),
        "command -v sha256sum >/dev/null".to_string(),
        "command -v date >/dev/null".to_string(),
        format!(
            "test \"$(git -C \"${{REPOSITORY_ROOT}}\" rev-parse HEAD)\" = {}",
            shell_quote(&spec.repository_commit)
        ),
        "test -z \"$(git -C \"${REPOSITORY_ROOT}\" status --porcelain --untracked-files=all)\""
            .to_string(),
        format!(
            "test \"$(sha256sum \"${{EXECUTABLE}}\" | awk '{{print $1}}')\" = {}",
            shell_quote(&spec.executable_sha256)
        ),
        format!(
            "test \"$(sha256sum \"${{APPTAINER}}\" | awk '{{print $1}}')\" = {}",
            shell_quote(&spec.apptainer_sha256)
        ),
        format!(
            "test \"$(sha256sum \"${{SPECIFICATION}}\" | awk '{{print $1}}')\" = {}",
            shell_quote(specification_sha256)
        ),
        "test ! -e \"${ATTEMPT_ROOT}/completion.json\" && test ! -L \"${ATTEMPT_ROOT}/completion.json\""
            .to_string(),
        "test ! -e \"${ATTEMPT_ROOT}/campaign-execution.json\" && test ! -L \"${ATTEMPT_ROOT}/campaign-execution.json\""
            .to_string(),
        command,
        String::new(),
    ]
    .join("\n")
}

fn ensure_attempt_root(spec: &ImageCampaignSlurmSpec) -> Result<PathBuf> {
    let mut current = spec.study_root.clone();
    for component in [
        "containers",
        "evidence",
        "image-campaign-submissions",
        &spec.repository_commit,
        "attempts",
        &spec.attempt_id,
    ] {
        current.push(component);
        ensure_directory(&current)?;
    }
    if current != spec.attempt_root {
        bail!("created image campaign attempt root differs from scheduler specification");
    }
    canonical_directory(&current, "image campaign attempt root")
}

fn ensure_directory(path: &Path) -> Result<PathBuf> {
    match fs::create_dir(path) {
        Ok(()) => {}
        Err(error) if error.kind() == ErrorKind::AlreadyExists => {}
        Err(error) => return Err(error).with_context(|| format!("create {}", path.display())),
    }
    canonical_directory(path, "image campaign scheduler directory")
}

fn ensure_bytes_identity(path: &Path, expected: &[u8]) -> Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                bail!("image campaign script is not a regular file");
            }
            if fs::read(path)? != expected {
                bail!("image campaign script conflicts with immutable specification");
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

fn require_utc_timestamp(value: &str) -> Result<()> {
    let parsed = chrono::DateTime::parse_from_rfc3339(value)
        .context("image campaign submission timestamp is invalid")?;
    if parsed.offset().local_minus_utc() != 0 || !value.ends_with('Z') {
        bail!("image campaign submission timestamp must use UTC Z notation");
    }
    Ok(())
}

fn shell_quote(value: impl AsRef<OsStr>) -> String {
    let value = value.as_ref().to_string_lossy();
    format!("'{}'", value.replace('\'', "'\\''"))
}
