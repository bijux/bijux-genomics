//! Owner: bijux-dna-runner
//! Failed qualification admission for owner-generic attempt archive transactions.

use std::path::Path;

use anyhow::{bail, Context, Result};
use bijux_dna_containers::api::load_container_governance;

use super::contracts::{
    ContainerQualificationAccountingRecord, ContainerQualificationSchedulerEvidence,
    ContainerQualificationSchedulerStatus, ContainerQualificationSlurmSpec,
    CONTAINER_QUALIFICATION_ACCOUNTING_SCHEMA_V1, CONTAINER_QUALIFICATION_SLURM_SPEC_SCHEMA_V5,
};
use super::lifecycle::{load_container_qualification_slurm_job_handle, validate_retained_evidence};
use super::validation::{
    canonical_directory, read_json, require_utc_timestamp, validate_recorded_specification,
};

/// Move one terminal failed qualification attempt through the shared container archive contract.
///
/// # Errors
/// Returns an error for historical layouts, non-failed evidence, identity drift, or any shared
/// archive transaction failure.
pub fn archive_failed_container_qualification_attempt(
    spec: &ContainerQualificationSlurmSpec,
    archived_at_utc: &str,
) -> Result<bijux_dna_infra::AttemptArchivePaths> {
    require_utc_timestamp(archived_at_utc, "qualification archival timestamp")?;
    validate_recorded_specification(spec)?;
    require_current_owned_layout(spec)?;
    let handle = load_container_qualification_slurm_job_handle(spec)?;
    let evidence_path = spec.evidence_root.join("scheduler-evidence.json");
    let evidence: ContainerQualificationSchedulerEvidence = read_json(&evidence_path)?;
    validate_retained_evidence(spec, &handle, &evidence)?;
    if evidence.status != ContainerQualificationSchedulerStatus::Failed {
        bail!("only terminal failed qualification evidence can be archived");
    }

    let attempt_root = canonical_directory(&spec.evidence_root, "qualification attempt root")?;
    let result_root = canonical_directory(&spec.result_root, "qualification result root")?;
    if attempt_root != result_root {
        bail!("current qualification result evidence must share one governed attempt root");
    }
    let accounting_path = attempt_root.join("slurm-accounting.json");
    let accounting: ContainerQualificationAccountingRecord = read_json(&accounting_path)?;
    validate_accounting(spec, &evidence, &accounting)?;
    let mirrored: ContainerQualificationSchedulerEvidence =
        read_json(&attempt_root.join("qualification-scheduler-evidence.json"))?;
    if mirrored != evidence {
        bail!("result scheduler evidence conflicts with terminal submission evidence");
    }

    let governance = load_container_governance(&spec.governance_path)?;
    let containers_root =
        canonical_directory(&governance.paths.containers_root, "governed containers root")?;
    let owner_root = canonical_directory(
        &containers_root.join("evidence/attempts"),
        "qualification attempt owner root",
    )?;
    let identity = archive_identity(spec)?;
    require_expected_attempt_root(&owner_root, &identity, &attempt_root)?;
    bijux_dna_infra::archive_attempt_layout(
        &owner_root,
        bijux_dna_infra::AttemptEvidenceOwner::Containers,
        &identity,
        bijux_dna_infra::AttemptArchiveDisposition::Failed,
        archived_at_utc,
    )
    .context("archive failed qualification attempt")
}

/// Restore one failed current qualification archive through the shared rollback contract.
///
/// The owner root is explicit because the archived attempt contains the immutable governance input
/// that originally identified it.
///
/// # Errors
/// Returns an error for historical layouts, owner-root drift, changed archive bytes, or a
/// conflicting rollback receipt.
pub fn rollback_failed_container_qualification_attempt_archive(
    owner_root: &Path,
    spec: &ContainerQualificationSlurmSpec,
    archived_at_utc: &str,
    rolled_back_at_utc: &str,
) -> Result<bijux_dna_infra::AttemptArchivePaths> {
    require_utc_timestamp(archived_at_utc, "qualification archival timestamp")?;
    require_utc_timestamp(rolled_back_at_utc, "qualification rollback timestamp")?;
    require_current_owned_layout(spec)?;
    let owner_root = canonical_directory(owner_root, "qualification attempt owner root")?;
    let identity = archive_identity(spec)?;
    require_expected_attempt_root(&owner_root, &identity, &spec.evidence_root)?;
    bijux_dna_infra::rollback_attempt_archive(
        &owner_root,
        bijux_dna_infra::AttemptEvidenceOwner::Containers,
        &identity,
        bijux_dna_infra::AttemptArchiveDisposition::Failed,
        archived_at_utc,
        rolled_back_at_utc,
    )
    .context("roll back failed qualification attempt archive")
}

fn archive_identity(
    spec: &ContainerQualificationSlurmSpec,
) -> Result<bijux_dna_infra::AttemptLayoutIdentity> {
    Ok(bijux_dna_infra::AttemptLayoutIdentity {
        run_directory: spec.run_id.clone(),
        tool_id: spec.tool_id.clone(),
        campaign_id: spec
            .campaign_id
            .clone()
            .context("current qualification archive omits campaign identity")?,
        execution_unit_id: None,
        attempt_id: spec
            .attempt_id
            .clone()
            .context("current qualification archive omits attempt identity")?,
    })
}

fn require_current_owned_layout(spec: &ContainerQualificationSlurmSpec) -> Result<()> {
    if spec.schema_version != CONTAINER_QUALIFICATION_SLURM_SPEC_SCHEMA_V5 {
        bail!("only current owner-scoped qualification attempts can be archived");
    }
    if spec.result_root != spec.evidence_root {
        bail!("current qualification result evidence must share one governed attempt root");
    }
    Ok(())
}

fn require_expected_attempt_root(
    owner_root: &Path,
    identity: &bijux_dna_infra::AttemptLayoutIdentity,
    observed: &Path,
) -> Result<()> {
    let expected = bijux_dna_infra::attempt_layout_paths(owner_root, identity)
        .context("derive qualification attempt archive path")?;
    if expected.attempt_dir != observed {
        bail!("qualification attempt path differs from its permanent identity");
    }
    Ok(())
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
