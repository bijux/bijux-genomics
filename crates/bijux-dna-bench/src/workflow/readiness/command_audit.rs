//! Owner: audit admitted command observations and publish their diagnostic result.
use crate::repo::command_audit as repository;
use anyhow::{bail, Result};
use bijux_dna_bench_model::{
    AllDomainNoPlaceholderCommandCheckReport, AllDomainRenderedCommandsReport,
};
use std::path::Path;

/// Publish diagnostics before returning a negative command audit verdict.
///
/// # Errors
/// Rejects malformed observations, failed publication or a diagnostic with findings.
pub fn write_recorded_command_audit(
    repo_root: &Path,
    requested_output: &Path,
    observed: &AllDomainRenderedCommandsReport,
) -> Result<AllDomainNoPlaceholderCommandCheckReport> {
    let path = repository::output_path(repo_root, requested_output);
    let report = bijux_dna_bench_model::evaluate_recorded_command_audit(
        observed,
        repository::display_path(repo_root, &path),
    )?;
    repository::publish(&path, &report)?;
    if !report.ok {
        bail!("all-domain active commands still contain placeholder execution drift");
    }
    Ok(report)
}

#[cfg(test)]
#[path = "../../../tests/contracts/readiness/command_audit.rs"]
mod tests;
