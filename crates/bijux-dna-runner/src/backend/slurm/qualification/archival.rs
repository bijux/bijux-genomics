//! Owner: bijux-dna-runner
//! Immutable archival for failed container qualification attempts.

use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use anyhow::{bail, Context, Result};
use bijux_dna_containers::api::load_container_governance;
use walkdir::WalkDir;

use super::contracts::{
    ArchivedContainerQualificationAttempt, ContainerQualificationAccountingRecord,
    ContainerQualificationArchiveFile, ContainerQualificationArchiveSources,
    ContainerQualificationFailedAttemptArchive, ContainerQualificationSchedulerEvidence,
    ContainerQualificationSchedulerStatus, ContainerQualificationSlurmSpec,
    CONTAINER_QUALIFICATION_ACCOUNTING_SCHEMA_V1,
    CONTAINER_QUALIFICATION_FAILED_ATTEMPT_ARCHIVE_SCHEMA_V1,
};
use super::lifecycle::{load_container_qualification_slurm_job_handle, validate_retained_evidence};
use super::validation::{
    canonical_directory, read_json, require_utc_timestamp, validate_specification,
};

/// Move one terminal failed qualification attempt into its immutable governed archive.
///
/// The submission and result roots are moved through a recoverable staging directory. They must
/// share a filesystem with the governed failure archive, so publication never degrades to a copy.
///
/// # Errors
/// Returns an error for non-failed or conflicting evidence, symlinks, cross-filesystem paths,
/// existing archive identities, or an incomplete filesystem move.
pub fn archive_failed_container_qualification_attempt(
    spec: &ContainerQualificationSlurmSpec,
    archived_at_utc: &str,
) -> Result<ArchivedContainerQualificationAttempt> {
    require_utc_timestamp(archived_at_utc, "qualification archival timestamp")?;
    validate_specification(spec)?;
    let handle = load_container_qualification_slurm_job_handle(spec)?;
    let evidence_path = spec.evidence_root.join("scheduler-evidence.json");
    let evidence: ContainerQualificationSchedulerEvidence = read_json(&evidence_path)?;
    validate_retained_evidence(spec, &handle, &evidence)?;
    if evidence.status != ContainerQualificationSchedulerStatus::Failed {
        bail!("only terminal failed qualification evidence can be archived");
    }

    let result_root = canonical_directory(&spec.result_root, "qualification result root")?;
    let accounting_path = result_root.join("slurm-accounting.json");
    let accounting: ContainerQualificationAccountingRecord = read_json(&accounting_path)?;
    validate_accounting(spec, &evidence, &accounting)?;
    let mirrored: ContainerQualificationSchedulerEvidence =
        read_json(&result_root.join("qualification-scheduler-evidence.json"))?;
    if mirrored != evidence {
        bail!("result scheduler evidence conflicts with terminal submission evidence");
    }

    let governance = load_container_governance(&spec.governance_path)?;
    let study_root = canonical_directory(&governance.paths.study_root, "governed study root")?;
    let containers_root =
        canonical_directory(&governance.paths.containers_root, "governed containers root")?;
    let submission_root = canonical_directory(&spec.evidence_root, "qualification evidence root")?;
    let archive_root = containers_root.join("evidence/failures");
    fs::create_dir_all(&archive_root)
        .with_context(|| format!("create qualification archive root {}", archive_root.display()))?;
    let archive_root = canonical_directory(&archive_root, "qualification archive root")?;
    require_same_filesystem(&submission_root, &result_root, &archive_root)?;

    let bundle_name = format!("{}-{}-slurm-{}", spec.run_id, spec.tool_id, handle.job_id);
    let target = archive_root.join(&bundle_name);
    let staging = archive_root.join(format!(".{bundle_name}.partial"));
    require_absent(&target, "qualification attempt archive")?;
    require_absent(&staging, "qualification attempt archive staging directory")?;

    let mut files = archive_file_records(&submission_root, "submission")?;
    files.extend(archive_file_records(&result_root, "result")?);
    let source_paths = ContainerQualificationArchiveSources {
        submission: study_uri(&study_root, &submission_root)?,
        result: study_uri(&study_root, &result_root)?,
    };
    let archive = ContainerQualificationFailedAttemptArchive {
        schema_version: CONTAINER_QUALIFICATION_FAILED_ATTEMPT_ARCHIVE_SCHEMA_V1.to_string(),
        run_id: spec.run_id.clone(),
        tool_id: spec.tool_id.clone(),
        slurm_job_id: handle.job_id,
        archived_at_utc: archived_at_utc.to_string(),
        source_paths,
        scheduler_evidence_sha256: bijux_dna_infra::hash_file_sha256(&evidence_path)?,
        accounting_sha256: bijux_dna_infra::hash_file_sha256(&accounting_path)?,
        files,
    };
    publish_archive(&submission_root, &result_root, &staging, &target, &archive)?;
    Ok(ArchivedContainerQualificationAttempt { path: target, archive })
}

fn validate_accounting(
    spec: &ContainerQualificationSlurmSpec,
    evidence: &ContainerQualificationSchedulerEvidence,
    accounting: &ContainerQualificationAccountingRecord,
) -> Result<()> {
    if accounting.schema_version != CONTAINER_QUALIFICATION_ACCOUNTING_SCHEMA_V1
        || accounting.run_id != spec.run_id
        || accounting.tool_id != spec.tool_id
        || accounting.slurm_job_id != evidence.job_id
        || accounting.accounting != evidence.accounting
        || accounting.specification_sha256 != evidence.specification_sha256
        || accounting.script_sha256 != evidence.script_sha256
        || accounting.governance_sha256 != evidence.governance_sha256
        || accounting.pipeline_sha256 != evidence.pipeline_sha256
        || accounting.credential_reference_sha256 != evidence.credential_reference_sha256
        || accounting.collected_at_utc != evidence.collected_at_utc
    {
        bail!("Slurm accounting does not identify the terminal qualification attempt");
    }
    Ok(())
}

fn archive_file_records(
    root: &Path,
    scope: &str,
) -> Result<Vec<ContainerQualificationArchiveFile>> {
    let mut records = Vec::new();
    for entry in WalkDir::new(root).sort_by_file_name() {
        let entry = entry.with_context(|| format!("walk archived evidence {}", root.display()))?;
        let metadata = fs::symlink_metadata(entry.path())
            .with_context(|| format!("inspect archived evidence {}", entry.path().display()))?;
        if metadata.file_type().is_symlink() {
            bail!("archived qualification evidence contains a symlink: {}", entry.path().display());
        }
        if metadata.is_file() {
            let relative =
                entry.path().strip_prefix(root).context("derive archive inventory path")?;
            records.push(ContainerQualificationArchiveFile {
                path: format!("{scope}/{}", relative.to_string_lossy().replace('\\', "/")),
                size_bytes: metadata.len(),
                sha256: bijux_dna_infra::hash_file_sha256(entry.path())?,
            });
        }
    }
    Ok(records)
}

fn require_same_filesystem(left: &Path, right: &Path, archive_root: &Path) -> Result<()> {
    if !bijux_dna_infra::same_filesystem(left, right)?
        || !bijux_dna_infra::same_filesystem(left, archive_root)?
    {
        bail!("qualification attempt evidence and archive must remain on one filesystem");
    }
    Ok(())
}

fn study_uri(study_root: &Path, path: &Path) -> Result<String> {
    let relative = path.strip_prefix(study_root).with_context(|| {
        format!("attempt evidence is outside governed STUDY: {}", path.display())
    })?;
    Ok(format!("study://{}", relative.to_string_lossy().replace('\\', "/")))
}

fn require_absent(path: &Path, label: &str) -> Result<()> {
    match fs::symlink_metadata(path) {
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Ok(_) => bail!("refusing to replace {label}: {}", path.display()),
        Err(error) => Err(error).with_context(|| format!("inspect {label}")),
    }
}

fn publish_archive(
    submission_root: &Path,
    result_root: &Path,
    staging: &Path,
    target: &Path,
    archive: &ContainerQualificationFailedAttemptArchive,
) -> Result<()> {
    fs::create_dir(staging)
        .with_context(|| format!("create archive staging directory {}", staging.display()))?;
    let operation = || -> Result<()> {
        fs::rename(submission_root, staging.join("submission"))?;
        fs::rename(result_root, staging.join("result"))?;
        bijux_dna_infra::atomic_create_json(&staging.join("archive.json"), archive)?;
        fs::rename(staging, target)?;
        Ok(())
    };
    operation().with_context(|| {
        format!(
            "qualification archival stopped; inspect recoverable staging path {}",
            staging.display()
        )
    })
}
