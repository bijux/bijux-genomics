use super::declarations::FASTQ_RENDERED_COMMANDS_SCHEMA_VERSION;
use super::format::render_fastq_command_argv_jsonl;
use super::format::render_fastq_commands_shell_script;
use super::models::FastqRenderedCommandsReport;
use super::paths::companion_argv_output_path;
use super::paths::path_relative_to_repo;
use super::paths::repo_relative_path;
use super::paths::repo_root_relative_to_output;
use super::projection::collect_fastq_rendered_command_rows;
use crate::readiness::declarations::CommandContext;
use anyhow::Context;
use anyhow::Result;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::Path;
/// Publish canonical FASTQ declaration evidence.
///
/// # Errors
/// Returns source admission, contract or publication failure.
pub fn render_fastq_commands(
    context: &CommandContext<'_>,
    output_path: &Path,
) -> Result<FastqRenderedCommandsReport> {
    let repo_root = context.repo_root;
    let output_path = repo_relative_path(repo_root, output_path);
    let argv_output_path = companion_argv_output_path(&output_path);
    let rows = collect_fastq_rendered_command_rows(context)?;

    if let Some(parent) = output_path.parent() {
        bijux_dna_infra::ensure_dir(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }
    if let Some(parent) = argv_output_path.parent() {
        bijux_dna_infra::ensure_dir(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }

    let rendered_script = render_fastq_commands_shell_script(
        &rows,
        &repo_root_relative_to_output(repo_root, &output_path),
    );
    bijux_dna_infra::atomic_write_bytes(&output_path, rendered_script.as_bytes())?;
    let argv_jsonl =
        render_fastq_command_argv_jsonl(&rows).context("render FASTQ command argv JSONL")?;
    bijux_dna_infra::atomic_write_bytes(&argv_output_path, argv_jsonl.as_bytes())?;

    let mut command_source_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *command_source_counts.entry(row.command_source.clone()).or_default() += 1;
    }

    Ok(FastqRenderedCommandsReport {
        schema_version: FASTQ_RENDERED_COMMANDS_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        argv_output_path: path_relative_to_repo(repo_root, &argv_output_path),
        row_count: rows.len(),
        stage_count: rows.iter().map(|row| row.stage_id.as_str()).collect::<BTreeSet<_>>().len(),
        tool_count: rows.iter().map(|row| row.tool_id.as_str()).collect::<BTreeSet<_>>().len(),
        command_source_counts,
        rows,
    })
}
