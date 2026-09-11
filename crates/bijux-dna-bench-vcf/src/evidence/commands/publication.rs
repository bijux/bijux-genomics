use super::paths::{
    companion_argv_output_path, path_relative_to_repo, repo_relative_path,
    repo_root_relative_to_output,
};
use super::{
    collect_vcf_rendered_command_rows_with_observations, VCF_RENDERED_COMMANDS_SCHEMA_VERSION,
};
use crate::adapters::observations::VcfAdapterObservations;
use anyhow::{bail, Context, Result};
use bijux_dna_bench_vcf_evidence::{
    commands::projection::{render_vcf_command_argv_jsonl, render_vcf_commands_shell_script},
    model::command_reports::VcfRenderedCommandsReport,
};
use std::path::Path;
/// Publish compatible shell and structured argv artifacts from admitted observations.
///
/// # Errors
/// Refuses inconsistent request roots or identities, incomplete command coverage and publication failures.
pub fn render_vcf_commands_with_observations(
    repo_root: &Path,
    output_path: &Path,
    observations: &VcfAdapterObservations<'_>,
    retained: &[bijux_dna_bench_vcf_evidence::model::scope::VcfActiveStageToolMatrixRow],
) -> Result<VcfRenderedCommandsReport> {
    anyhow::ensure!(
        repo_root == observations.repo_root(),
        "VCF command publication root disagrees with its observations"
    );
    let output_path = repo_relative_path(repo_root, output_path);
    let argv_output_path = companion_argv_output_path(&output_path);
    let rows = collect_vcf_rendered_command_rows_with_observations(observations, retained)?;
    let active_row_count = retained.iter().filter(|row| row.scope_state == "active").count();
    if rows.len() != active_row_count {
        bail!(
            "VCF rendered commands must cover every active VCF row (expected {active_row_count}, found {})",
            rows.len()
        );
    }

    let rendered_script = render_vcf_commands_shell_script(
        &rows,
        &repo_root_relative_to_output(repo_root, &output_path),
    );
    let argv_jsonl =
        render_vcf_command_argv_jsonl(&rows).context("render VCF command argv JSONL")?;
    bijux_dna_infra::atomic_write_bytes(&output_path, rendered_script.as_bytes())?;
    bijux_dna_infra::atomic_write_bytes(&argv_output_path, argv_jsonl.as_bytes())?;

    Ok(VcfRenderedCommandsReport {
        schema_version: VCF_RENDERED_COMMANDS_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        argv_output_path: path_relative_to_repo(repo_root, &argv_output_path),
        row_count: rows.len(),
        rows,
    })
}
