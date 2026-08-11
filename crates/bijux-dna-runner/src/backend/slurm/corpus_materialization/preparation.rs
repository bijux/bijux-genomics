//! Owner: bijux-dna-runner
//! Immutable Slurm material preparation for corpus acquisition.

use std::ffi::OsStr;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

use super::contracts::{
    CorpusMaterializationSlurmSpec, PreparedCorpusMaterializationSlurmJob,
    CORPUS_MATERIALIZATION_PREPARED_JOB_SCHEMA_V1,
};
use super::validation::{
    canonical_directory, catalog_snapshot_path, ensure_json_identity, require_utc_timestamp,
    validate_catalog_precondition, validate_command_arguments, validate_specification,
};
use crate::backend::slurm::{SlurmDependencyCondition, SlurmSubmitRequest};

/// Validate exact source authorities and publish restart-safe materialization scheduler files.
///
/// # Errors
/// Returns an error for authority drift, unsafe paths, conflicting evidence, invalid resources,
/// catalog precondition drift, or an invalid submission timestamp.
pub fn prepare_corpus_materialization_slurm_job(
    spec: &CorpusMaterializationSlurmSpec,
    submitted_at_utc: &str,
) -> Result<PreparedCorpusMaterializationSlurmJob> {
    validate_specification(spec)?;
    validate_command_arguments(&spec.command_arguments)?;
    validate_catalog_precondition(spec)?;
    require_utc_timestamp(submitted_at_utc, "submission timestamp")?;

    let evidence_root = ensure_evidence_root(&spec.evidence_root)?;
    let specification_path = evidence_root.join("materialization-spec.json");
    ensure_json_identity(&specification_path, spec)?;
    let specification_sha256 = bijux_dna_infra::hash_file_sha256(&specification_path)
        .context("hash corpus materialization Slurm specification")?;

    let scheduler_root = ensure_directory(&evidence_root.join("scheduler"))?;
    let script_path = scheduler_root.join("job.sbatch");
    let script = render_script(spec, &specification_sha256);
    ensure_bytes_identity(&script_path, script.as_bytes())?;
    make_executable(&script_path)?;
    let script_sha256 =
        bijux_dna_infra::hash_file_sha256(&script_path).context("hash materialization script")?;

    let prepared = PreparedCorpusMaterializationSlurmJob {
        schema_version: CORPUS_MATERIALIZATION_PREPARED_JOB_SCHEMA_V1.to_string(),
        request: SlurmSubmitRequest {
            run_id: format!("{}:{}:{}", spec.run_id, spec.portfolio_id, spec.attempt_id),
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

fn render_script(spec: &CorpusMaterializationSlurmSpec, specification_sha256: &str) -> String {
    let scheduler_root = spec.evidence_root.join("scheduler");
    let publication = spec.evidence_root.join("materialization-publication.json");
    let completion = scheduler_root.join("completed.json");
    let catalog = catalog_snapshot_path(&spec.data_root, spec);
    let catalog_precondition = spec.existing_catalog_sha256.as_ref().map_or_else(
        || "test ! -e \"${CATALOG}\" && test ! -L \"${CATALOG}\"".to_string(),
        |sha256| {
            format!(
                "test -f \"${{CATALOG}}\" && test ! -L \"${{CATALOG}}\" && test \"$(sha256sum \"${{CATALOG}}\" | awk '{{print $1}}')\" = {}",
                shell_quote(sha256)
            )
        },
    );
    let command = std::iter::once(shell_quote(&spec.executable_path))
        .chain(spec.command_arguments.iter().map(shell_quote))
        .collect::<Vec<_>>()
        .join(" ");
    [
        "#!/usr/bin/env bash".to_string(),
        format!("#SBATCH --job-name={}", job_name(spec)),
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
        format!("readonly PLAN={}", shell_quote(&spec.plan_path)),
        format!(
            "readonly SPECIFICATION={}",
            shell_quote(spec.evidence_root.join("materialization-spec.json"))
        ),
        format!("readonly PUBLICATION={}", shell_quote(&publication)),
        format!("readonly COMPLETION={}", shell_quote(&completion)),
        format!("readonly CATALOG={}", shell_quote(&catalog)),
        format!("mkdir {}", shell_quote(scheduler_root.join("execution-lock"))),
        "command -v git >/dev/null".to_string(),
        "command -v sha256sum >/dev/null".to_string(),
        "command -v awk >/dev/null".to_string(),
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
            "test \"$(sha256sum \"${{PLAN}}\" | awk '{{print $1}}')\" = {}",
            shell_quote(&spec.plan_sha256)
        ),
        format!(
            "test \"$(sha256sum \"${{SPECIFICATION}}\" | awk '{{print $1}}')\" = {}",
            shell_quote(specification_sha256)
        ),
        catalog_precondition,
        "test ! -e \"${PUBLICATION}\" && test ! -L \"${PUBLICATION}\""
            .to_string(),
        format!("{command} > \"${{PUBLICATION}}.partial\""),
        "test -s \"${PUBLICATION}.partial\"".to_string(),
        "mv \"${PUBLICATION}.partial\" \"${PUBLICATION}\"".to_string(),
        "test -f \"${CATALOG}\" && test ! -L \"${CATALOG}\"".to_string(),
        format!(
            "printf '{{\"schema_version\":\"bijux.runner.corpus_materialization_completion.v1\",\"specification_sha256\":\"{specification_sha256}\"}}\\n' > \"${{COMPLETION}}.partial\""
        ),
        "mv \"${COMPLETION}.partial\" \"${COMPLETION}\"".to_string(),
        String::new(),
    ]
    .join("\n")
}

fn ensure_evidence_root(path: &Path) -> Result<PathBuf> {
    match fs::create_dir(path) {
        Ok(()) => {}
        Err(error) if error.kind() == ErrorKind::AlreadyExists => {}
        Err(error) => return Err(error).with_context(|| format!("create {}", path.display())),
    }
    canonical_directory(path, "corpus materialization evidence root")
}

fn ensure_directory(path: &Path) -> Result<PathBuf> {
    match fs::create_dir(path) {
        Ok(()) => {}
        Err(error) if error.kind() == ErrorKind::AlreadyExists => {}
        Err(error) => return Err(error).with_context(|| format!("create {}", path.display())),
    }
    canonical_directory(path, "corpus materialization scheduler directory")
}

fn ensure_bytes_identity(path: &Path, expected: &[u8]) -> Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                bail!("materialization script is not a regular file");
            }
            if fs::read(path).with_context(|| format!("read {}", path.display()))? != expected {
                bail!("materialization script conflicts with immutable specification");
            }
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {
            bijux_dna_infra::atomic_create_bytes(path, expected)
                .with_context(|| format!("publish {}", path.display()))?;
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

fn job_name(spec: &CorpusMaterializationSlurmSpec) -> String {
    let suffix = spec.portfolio_id.chars().take(28).collect::<String>();
    format!("bijux-data-{}-{suffix}", spec.run_id)
}

fn shell_quote(value: impl AsRef<OsStr>) -> String {
    let value = value.as_ref().to_string_lossy();
    format!("'{}'", value.replace('\'', "'\\''"))
}
