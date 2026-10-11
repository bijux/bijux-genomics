//! Owner: admit and render both command representations before repository publication.
use crate::artifacts::command_artifacts::{
    render_all_domain_command_argv_jsonl, render_all_domain_commands_shell_script,
};
use crate::repo::command_artifacts as repository;
use anyhow::{Context, Result};
use bijux_dna_bench_model::{
    build_rendered_command_artifact_report, AllDomainRenderedCommandRow,
    AllDomainRenderedCommandsReport,
};
use std::path::Path;

/// Publish shell and argv projections for intrinsically admitted command observations.
/// The caller remains responsible for matching the complete governed result slice.
///
/// # Errors
/// Rejects invalid paths, malformed observations, serialization failures or file IO failures.
/// Each file is replaced atomically; the pair is not a filesystem transaction.
pub fn publish_rendered_command_artifacts(
    repo_root: &Path,
    requested: &Path,
    rows: Vec<AllDomainRenderedCommandRow>,
) -> Result<AllDomainRenderedCommandsReport> {
    let paths = repository::prepare(repo_root, requested)?;
    let report = build_rendered_command_artifact_report(
        rows,
        paths.script_display.clone(),
        paths.argv_display.clone(),
    )?;
    let script = render_all_domain_commands_shell_script(&report.rows, &paths.repo_root_ascent);
    let argv = render_all_domain_command_argv_jsonl(&report.rows)
        .context("render all-domain command argv JSONL")?;
    repository::publish(&paths, &script, &argv)?;
    Ok(report)
}

#[cfg(test)]
#[path = "../../../tests/contracts/readiness/command_artifacts.rs"]
mod tests;
