use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Serialize;

use super::vcf_rendered_command_rows::{collect_vcf_rendered_command_rows, VcfRenderedCommandRow};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_VCF_RENDERED_COMMANDS_PATH: &str =
    "benchmarks/readiness/vcf-rendered-commands.sh";
const VCF_RENDERED_COMMANDS_SCHEMA_VERSION: &str = "bijux.bench.readiness.vcf_rendered_commands.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct VcfRenderedCommandArgvRow {
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) readiness_kind: String,
    pub(crate) benchmark_status: String,
    pub(crate) command_source: String,
    pub(crate) command_steps: Vec<VcfRenderedCommandArgvStep>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct VcfRenderedCommandArgvStep {
    pub(crate) step_id: String,
    pub(crate) step_kind: String,
    pub(crate) consumes_previous_stdout: bool,
    pub(crate) argv: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct VcfRenderedCommandsReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) argv_output_path: String,
    pub(crate) row_count: usize,
    pub(crate) rows: Vec<VcfRenderedCommandRow>,
}

pub(crate) fn run_render_vcf_commands(
    args: &parse::BenchReadinessRenderVcfCommandsArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_vcf_commands(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_RENDERED_COMMANDS_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_vcf_commands(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<VcfRenderedCommandsReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let argv_output_path = companion_argv_output_path(&output_path);
    let rows = collect_vcf_rendered_command_rows(repo_root)?;

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    if let Some(parent) = argv_output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    let rendered_script = render_vcf_commands_shell_script(&rows);
    bijux_dna_infra::atomic_write_bytes(&output_path, rendered_script.as_bytes())?;
    let argv_jsonl =
        render_vcf_command_argv_jsonl(&rows).context("render VCF command argv JSONL")?;
    bijux_dna_infra::atomic_write_bytes(&argv_output_path, argv_jsonl.as_bytes())?;

    Ok(VcfRenderedCommandsReport {
        schema_version: VCF_RENDERED_COMMANDS_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        argv_output_path: path_relative_to_repo(repo_root, &argv_output_path),
        row_count: rows.len(),
        rows,
    })
}

fn render_vcf_commands_shell_script(rows: &[VcfRenderedCommandRow]) -> String {
    let mut rendered = String::from("#!/usr/bin/env bash\nset -euo pipefail\n");
    rendered.push_str("repo_root=\"$(cd \"$(dirname \"${BASH_SOURCE[0]}\")/../..\" && pwd)\"\n");
    rendered.push_str("cd \"$repo_root\"\n\n");
    for (index, row) in rows.iter().enumerate() {
        rendered.push_str(&format!("# {} / {}\n", row.stage_id, row.tool_id));
        for command in &row.script_commands {
            rendered.push_str(command);
            rendered.push('\n');
        }
        if index + 1 < rows.len() {
            rendered.push('\n');
        }
    }
    rendered
}

fn render_vcf_command_argv_jsonl(rows: &[VcfRenderedCommandRow]) -> Result<String> {
    let mut rendered = String::new();
    for row in rows {
        let payload = VcfRenderedCommandArgvRow {
            stage_id: row.stage_id.clone(),
            tool_id: row.tool_id.clone(),
            readiness_kind: row.readiness_kind.clone(),
            benchmark_status: row.benchmark_status.clone(),
            command_source: row.command_source.clone(),
            command_steps: row
                .command_steps
                .iter()
                .map(|step| VcfRenderedCommandArgvStep {
                    step_id: step.step_id.clone(),
                    step_kind: step.step_kind.clone(),
                    consumes_previous_stdout: step.consumes_previous_stdout,
                    argv: step.argv.clone(),
                })
                .collect(),
        };
        let line =
            serde_json::to_string(&payload).context("serialize VCF rendered command argv row")?;
        rendered.push_str(&line);
        rendered.push('\n');
    }
    Ok(rendered)
}

fn companion_argv_output_path(output_path: &Path) -> PathBuf {
    let file_name = output_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(DEFAULT_VCF_RENDERED_COMMANDS_PATH);
    let argv_name = if let Some(stem) = file_name.strip_suffix(".sh") {
        format!("{stem}.argv.jsonl")
    } else {
        format!("{file_name}.argv.jsonl")
    };
    output_path.with_file_name(argv_name)
}

fn repo_relative_path(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{render_vcf_commands, DEFAULT_VCF_RENDERED_COMMANDS_PATH};

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn vcf_rendered_commands_report_tracks_canonical_rows() {
        let root = repo_root();
        let report = render_vcf_commands(&root, PathBuf::from(DEFAULT_VCF_RENDERED_COMMANDS_PATH))
            .expect("render VCF commands");

        assert_eq!(report.schema_version, "bijux.bench.readiness.vcf_rendered_commands.v1");
        assert_eq!(report.output_path, "benchmarks/readiness/vcf-rendered-commands.sh");
        assert_eq!(
            report.argv_output_path,
            "benchmarks/readiness/vcf-rendered-commands.argv.jsonl"
        );
        assert_eq!(report.row_count, 12);
        assert!(report.rows.iter().all(|row| row.tool_id == "bcftools"));
    }
}
