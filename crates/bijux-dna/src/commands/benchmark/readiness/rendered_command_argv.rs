use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Serialize;

use crate::commands::benchmark::local_stage_commands::collect_local_stage_command_entries;
use crate::commands::benchmark::local_stage_inventory::LocalStageReadinessKind;
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_RENDERED_COMMAND_ARGV_PATH: &str =
    "target/bench-readiness/rendered-commands.argv.jsonl";
const RENDERED_COMMAND_ARGV_SCHEMA_VERSION: &str = "bijux.bench.readiness.rendered_command_argv.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct RenderedCommandArgvRow {
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) readiness_kind: String,
    pub(crate) argv: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct RenderedCommandArgvReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) row_count: usize,
    pub(crate) rows: Vec<RenderedCommandArgvRow>,
}

pub(crate) fn run_render_command_argv(
    args: &parse::BenchReadinessRenderCommandArgvArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_command_argv(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_RENDERED_COMMAND_ARGV_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_command_argv(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<RenderedCommandArgvReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let rows = collect_local_stage_command_entries(repo_root, None)?
        .into_iter()
        .map(|entry| RenderedCommandArgvRow {
            stage_id: entry.stage_id,
            tool_id: entry.tool_id,
            readiness_kind: readiness_kind_label(entry.readiness_kind).to_string(),
            argv: entry.argv,
        })
        .collect::<Vec<_>>();

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let rendered = render_command_argv_jsonl(&rows).context("render command argv JSONL")?;
    bijux_dna_infra::atomic_write_bytes(&output_path, rendered.as_bytes())?;

    Ok(RenderedCommandArgvReport {
        schema_version: RENDERED_COMMAND_ARGV_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        row_count: rows.len(),
        rows,
    })
}

fn render_command_argv_jsonl(rows: &[RenderedCommandArgvRow]) -> Result<String> {
    let mut rendered = String::new();
    for row in rows {
        let line =
            serde_json::to_string(row).context("serialize rendered command argv row to JSON")?;
        rendered.push_str(&line);
        rendered.push('\n');
    }
    Ok(rendered)
}

fn readiness_kind_label(readiness_kind: LocalStageReadinessKind) -> &'static str {
    match readiness_kind {
        LocalStageReadinessKind::DryRun => "dry_run",
        LocalStageReadinessKind::Smoke => "smoke",
        LocalStageReadinessKind::DryOrSmoke => "dry_or_smoke",
    }
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
    fn rendered_command_argv_reports_governed_51_row_slice() {
        use super::{render_command_argv, DEFAULT_RENDERED_COMMAND_ARGV_PATH};

        let root = repo_root();
        let report = render_command_argv(&root, PathBuf::from(DEFAULT_RENDERED_COMMAND_ARGV_PATH))
            .expect("render command argv");

        assert_eq!(report.schema_version, "bijux.bench.readiness.rendered_command_argv.v1");
        assert_eq!(report.output_path, "target/bench-readiness/rendered-commands.argv.jsonl");
        assert_eq!(report.row_count, 51);
        assert_eq!(report.rows.len(), 51);
        assert!(report.rows.iter().all(|row| {
            row.argv.first().is_some_and(|arg| arg == "cargo")
                && row.argv.iter().any(|arg| arg == "--stage-id")
                && row.argv.last().is_some_and(|arg| arg == &row.stage_id)
        }));
    }
}
