use super::{SlurmSubmissionReport, HPC_SLURM_SUBMISSION_SCHEMA_VERSION};
use anyhow::{ensure, Result};
use std::collections::BTreeSet;

/// Admit report identity and exact ordered in-scope scheduler dependency closure.
/// # Errors
/// Rejects unsupported versions/modes, empty identities, duplicate jobs or unresolved dependencies.
pub fn validate_hpc_submission_report(report: &SlurmSubmissionReport) -> Result<()> {
    ensure!(
        report.schema_version == HPC_SLURM_SUBMISSION_SCHEMA_VERSION,
        "unsupported Slurm submission report schema"
    );
    ensure!(matches!(report.mode.as_str(), "mock" | "real"), "unsupported Slurm submission mode");
    ensure!(!report.jobs.is_empty(), "submission report has no jobs");
    ensure!(
        [report.campaign_id.as_str(), report.domain.as_str(), report.submitted_at.as_str()]
            .iter()
            .all(|value| !value.trim().is_empty()),
        "submission report has an empty campaign identity"
    );
    let mut names = BTreeSet::new();
    let mut planned = BTreeSet::new();
    let mut submitted = BTreeSet::new();
    for job in &report.jobs {
        ensure!(
            [
                job.job_name.as_str(),
                job.stage.as_str(),
                job.tool.as_str(),
                job.sample.as_str(),
                job.planned_job_id.as_str(),
                job.scheduler_job_id.as_str(),
                job.script_path.as_str(),
                job.log_path.as_str(),
                job.out_path.as_str(),
                job.err_path.as_str(),
                job.results_path.as_str(),
                job.code_path.as_str(),
            ]
            .iter()
            .all(|value| !value.trim().is_empty()),
            "submission job has an empty identity or output path"
        );
        ensure!(
            names.insert(job.job_name.as_str()) && planned.insert(job.planned_job_id.as_str()),
            "duplicate submitted campaign job"
        );
        validate_scheduler_identity(&report.mode, &job.scheduler_job_id)?;
        let mut dependencies = BTreeSet::new();
        for dependency in &job.dependency_scheduler_ids {
            ensure!(
                submitted.contains(dependency.as_str()) && dependencies.insert(dependency.as_str()),
                "submission dependency is repeated or has no preceding receipt"
            );
        }
        ensure!(
            submitted.insert(job.scheduler_job_id.as_str()),
            "duplicate scheduler receipt identity"
        );
    }
    Ok(())
}

pub(super) fn deserialize_version<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<&'static str, D::Error> {
    use serde::Deserialize;
    let version = String::deserialize(deserializer)?;
    if version != HPC_SLURM_SUBMISSION_SCHEMA_VERSION {
        return Err(serde::de::Error::custom("unsupported Slurm submission report schema"));
    }
    Ok(HPC_SLURM_SUBMISSION_SCHEMA_VERSION)
}

pub(super) fn version_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
    schemars::json_schema!({"type":"string","const":HPC_SLURM_SUBMISSION_SCHEMA_VERSION})
}

pub(super) fn deserialize_mode<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<String, D::Error> {
    use serde::Deserialize;
    let mode = String::deserialize(deserializer)?;
    if !matches!(mode.as_str(), "mock" | "real") {
        return Err(serde::de::Error::custom("unsupported Slurm submission mode"));
    }
    Ok(mode)
}

pub(super) fn mode_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
    schemars::json_schema!({"type":"string","enum":["mock","real"]})
}

/// Admit an external receipt before publishing operator files or encrypted payloads.
/// # Errors
/// Rejects malformed mode-specific identities and identities returned for a previous job.
pub fn validate_hpc_scheduler_receipt(
    mode: super::HpcSubmissionMode,
    identity: &str,
    preceding: &[String],
) -> Result<()> {
    let mode = match mode {
        super::HpcSubmissionMode::Mock => "mock",
        super::HpcSubmissionMode::Real => "real",
    };
    validate_scheduler_identity(mode, identity)?;
    ensure!(
        !preceding.iter().any(|prior| prior == identity),
        "duplicate scheduler receipt identity"
    );
    Ok(())
}

fn validate_scheduler_identity(mode: &str, identity: &str) -> Result<()> {
    let number =
        if mode == "mock" { identity.strip_prefix("mock-").unwrap_or_default() } else { identity };
    ensure!(
        !number.is_empty() && number.bytes().all(|byte| byte.is_ascii_digit()),
        "invalid scheduler identity for submission mode"
    );
    Ok(())
}
