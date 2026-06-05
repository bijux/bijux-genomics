use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Serialize;

use super::benchmark_command_rows::{collect_benchmark_command_rows, render_shell_command};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_RENDERED_COMMANDS_PATH: &str =
    "target/bench-readiness/rendered-commands.sh";
const RENDERED_COMMANDS_SCHEMA_VERSION: &str = "bijux.bench.readiness.rendered_commands.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct RenderedCommandRow {
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) readiness_kind: String,
    pub(crate) argv: Vec<String>,
    pub(crate) command: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct RenderedCommandsReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) row_count: usize,
    pub(crate) rows: Vec<RenderedCommandRow>,
}

pub(crate) fn run_render_commands(args: &parse::BenchReadinessRenderCommandsArgs) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_commands(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_RENDERED_COMMANDS_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_commands(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<RenderedCommandsReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let rows = collect_rendered_command_rows(repo_root)?;

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let rendered = render_commands_shell_script(&rows);
    bijux_dna_infra::atomic_write_bytes(&output_path, rendered.as_bytes())?;

    Ok(RenderedCommandsReport {
        schema_version: RENDERED_COMMANDS_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        row_count: rows.len(),
        rows,
    })
}

pub(crate) fn collect_rendered_command_rows(repo_root: &Path) -> Result<Vec<RenderedCommandRow>> {
    collect_benchmark_command_rows(repo_root)?
        .into_iter()
        .map(|entry| {
            let command = render_shell_command(&entry.argv);
            Ok(RenderedCommandRow {
                stage_id: entry.stage_id,
                tool_id: entry.tool_id,
                readiness_kind: entry.readiness_kind,
                argv: entry.argv,
                command,
            })
        })
        .collect()
}

fn render_commands_shell_script(rows: &[RenderedCommandRow]) -> String {
    let mut rendered = String::from("#!/usr/bin/env bash\nset -euo pipefail\n");
    rendered.push_str("repo_root=\"$(cd \"$(dirname \"${BASH_SOURCE[0]}\")/../..\" && pwd)\"\n");
    rendered.push_str("cd \"$repo_root\"\n\n");
    for row in rows {
        rendered.push_str(&format!("# {} / {}\n{}\n", row.stage_id, row.tool_id, row.command));
    }
    rendered
}

fn repo_relative_path(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).display().to_string()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[cfg(feature = "bam_downstream")]
    #[test]
    fn rendered_commands_report_governed_benchmark_ready_row_slice() {
        use super::{render_commands, DEFAULT_RENDERED_COMMANDS_PATH};

        let root = repo_root();
        let report = render_commands(&root, PathBuf::from(DEFAULT_RENDERED_COMMANDS_PATH))
            .expect("render commands");

        assert_eq!(report.schema_version, "bijux.bench.readiness.rendered_commands.v1");
        assert_eq!(report.output_path, "target/bench-readiness/rendered-commands.sh");
        assert_eq!(report.row_count, 110);
        assert_eq!(report.rows.len(), 110);
        assert!(report.rows.iter().all(|row| {
            row.argv.first().is_some()
                && !row.command.is_empty()
                && row.command == crate::commands::benchmark::readiness::benchmark_command_rows::render_shell_command(&row.argv)
        }));
    }
}
