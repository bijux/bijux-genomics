use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use super::fastq_active_stage_tool_matrix::collect_fastq_active_stage_tool_matrix_rows;
use super::rendered_commands::{collect_rendered_command_rows, RenderedCommandRow};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_FASTQ_RENDERED_COMMANDS_PATH: &str =
    "benchmarks/readiness/fastq/fastq-rendered-commands.sh";
const FASTQ_RENDERED_COMMANDS_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.fastq_rendered_commands.v1";
const COMMAND_SOURCE_FASTQ_BAM_ADAPTER: &str = "fastq_bam_command_adapter";
const BENCHMARK_READY_STATUS: &str = "benchmark_ready";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct BindingKey {
    stage_id: String,
    tool_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct FastqRenderedCommandStep {
    pub(crate) step_id: String,
    pub(crate) step_kind: String,
    pub(crate) consumes_previous_stdout: bool,
    pub(crate) argv: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct FastqRenderedCommandRow {
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) corpus_id: String,
    pub(crate) asset_profile_id: String,
    pub(crate) adapter_id: String,
    pub(crate) parser_id: String,
    pub(crate) schema_id: String,
    pub(crate) readiness_kind: String,
    pub(crate) benchmark_status: String,
    pub(crate) command_source: String,
    pub(crate) command_steps: Vec<FastqRenderedCommandStep>,
    pub(crate) script_commands: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct FastqRenderedCommandArgvRow {
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) corpus_id: String,
    pub(crate) asset_profile_id: String,
    pub(crate) adapter_id: String,
    pub(crate) parser_id: String,
    pub(crate) schema_id: String,
    pub(crate) readiness_kind: String,
    pub(crate) benchmark_status: String,
    pub(crate) command_source: String,
    pub(crate) command_steps: Vec<FastqRenderedCommandStep>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct FastqRenderedCommandsReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) argv_output_path: String,
    pub(crate) row_count: usize,
    pub(crate) stage_count: usize,
    pub(crate) tool_count: usize,
    pub(crate) command_source_counts: BTreeMap<String, usize>,
    pub(crate) rows: Vec<FastqRenderedCommandRow>,
}

pub(crate) fn run_render_fastq_commands(
    args: &parse::BenchReadinessRenderFastqCommandsArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_fastq_commands(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_FASTQ_RENDERED_COMMANDS_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_fastq_commands(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<FastqRenderedCommandsReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let argv_output_path = companion_argv_output_path(&output_path);
    let rows = collect_fastq_rendered_command_rows(repo_root)?;

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    if let Some(parent) = argv_output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
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

pub(crate) fn collect_fastq_rendered_command_rows(
    repo_root: &Path,
) -> Result<Vec<FastqRenderedCommandRow>> {
    let active_rows = collect_fastq_active_stage_tool_matrix_rows(repo_root)?.rows;
    let rendered_by_binding = collect_rendered_command_rows(repo_root)?
        .into_iter()
        .filter(|row| row.stage_id.starts_with("fastq."))
        .map(|row| (binding_key(&row.stage_id, &row.tool_id), row))
        .collect::<BTreeMap<_, _>>();

    let mut rows = Vec::with_capacity(active_rows.len());
    let mut seen_bindings = BTreeSet::new();
    for active_row in active_rows {
        let binding = binding_key(&active_row.stage_id, &active_row.tool_id);
        let rendered_row = rendered_by_binding.get(&binding).ok_or_else(|| {
            anyhow!(
                "FASTQ rendered commands are missing benchmark-ready command coverage for `{}` / `{}`",
                active_row.stage_id,
                active_row.tool_id
            )
        })?;
        if !seen_bindings.insert(binding.clone()) {
            return Err(anyhow!(
                "FASTQ rendered commands contain duplicate `{}` / `{}` bindings",
                active_row.stage_id,
                active_row.tool_id
            ));
        }
        rows.push(rendered_fastq_row(active_row, rendered_row));
    }

    if rendered_by_binding.len() != rows.len() {
        let active_bindings = rows
            .iter()
            .map(|row| binding_key(&row.stage_id, &row.tool_id))
            .collect::<BTreeSet<_>>();
        let unexpected = rendered_by_binding
            .keys()
            .filter(|binding| !active_bindings.contains(*binding))
            .map(|binding| format!("{}/{}", binding.stage_id, binding.tool_id))
            .collect::<Vec<_>>();
        return Err(anyhow!(
            "FASTQ rendered commands must keep exactly one row per active FASTQ binding, unexpected rows: {}",
            unexpected.join(", ")
        ));
    }

    rows.sort_by(|left, right| {
        left.stage_id
            .cmp(&right.stage_id)
            .then_with(|| left.tool_id.cmp(&right.tool_id))
    });
    ensure_fastq_rendered_command_contract(&rows)?;
    Ok(rows)
}

fn rendered_fastq_row(
    active_row: super::fastq_active_stage_tool_matrix::FastqActiveStageToolMatrixRow,
    rendered_row: &RenderedCommandRow,
) -> FastqRenderedCommandRow {
    let step = FastqRenderedCommandStep {
        step_id: "invoke".to_string(),
        step_kind: "exec".to_string(),
        consumes_previous_stdout: false,
        argv: rendered_row.argv.clone(),
    };
    FastqRenderedCommandRow {
        stage_id: active_row.stage_id,
        tool_id: active_row.tool_id,
        corpus_id: active_row.corpus_id,
        asset_profile_id: active_row.asset_profile_id,
        adapter_id: active_row.adapter_id,
        parser_id: active_row.parser_id,
        schema_id: active_row.schema_id,
        readiness_kind: rendered_row.readiness_kind.clone(),
        benchmark_status: BENCHMARK_READY_STATUS.to_string(),
        command_source: COMMAND_SOURCE_FASTQ_BAM_ADAPTER.to_string(),
        command_steps: vec![step],
        script_commands: vec![rendered_row.command.clone()],
    }
}

fn ensure_fastq_rendered_command_contract(rows: &[FastqRenderedCommandRow]) -> Result<()> {
    let mut seen_bindings = BTreeSet::<BindingKey>::new();
    for row in rows {
        if row.stage_id.trim().is_empty()
            || row.tool_id.trim().is_empty()
            || row.corpus_id.trim().is_empty()
            || row.asset_profile_id.trim().is_empty()
            || row.adapter_id.trim().is_empty()
            || row.parser_id.trim().is_empty()
            || row.schema_id.trim().is_empty()
            || row.readiness_kind.trim().is_empty()
            || row.benchmark_status.trim().is_empty()
            || row.command_source.trim().is_empty()
        {
            return Err(anyhow!(
                "FASTQ rendered command row `{}` / `{}` is missing required columns",
                row.stage_id,
                row.tool_id
            ));
        }
        if row.benchmark_status != BENCHMARK_READY_STATUS {
            return Err(anyhow!(
                "FASTQ rendered command row `{}` / `{}` drifted from the benchmark-ready slice",
                row.stage_id,
                row.tool_id
            ));
        }
        if row.command_source != COMMAND_SOURCE_FASTQ_BAM_ADAPTER {
            return Err(anyhow!(
                "FASTQ rendered command row `{}` / `{}` drifted from the FASTQ/BAM adapter source",
                row.stage_id,
                row.tool_id
            ));
        }
        if row.command_steps.len() != 1 || row.script_commands.len() != 1 {
            return Err(anyhow!(
                "FASTQ rendered command row `{}` / `{}` must preserve exactly one command step and one shell command",
                row.stage_id,
                row.tool_id
            ));
        }
        let step = &row.command_steps[0];
        if step.step_id != "invoke" || step.step_kind != "exec" || step.argv.is_empty() {
            return Err(anyhow!(
                "FASTQ rendered command row `{}` / `{}` emitted an invalid invoke step",
                row.stage_id,
                row.tool_id
            ));
        }
        if row.script_commands[0].trim().is_empty() {
            return Err(anyhow!(
                "FASTQ rendered command row `{}` / `{}` emitted an empty shell command",
                row.stage_id,
                row.tool_id
            ));
        }
        if !seen_bindings.insert(binding_key(&row.stage_id, &row.tool_id)) {
            return Err(anyhow!(
                "FASTQ rendered commands contain duplicate `{}` / `{}` bindings",
                row.stage_id,
                row.tool_id
            ));
        }
    }
    Ok(())
}

fn render_fastq_commands_shell_script(
    rows: &[FastqRenderedCommandRow],
    repo_root_relative_to_output: &str,
) -> String {
    let mut rendered = String::from("#!/usr/bin/env bash\nset -euo pipefail\n");
    rendered.push_str(&format!(
        "repo_root=\"$(cd \"$(dirname \"${{BASH_SOURCE[0]}}\")/{}\" && pwd)\"\n",
        repo_root_relative_to_output
    ));
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

fn render_fastq_command_argv_jsonl(rows: &[FastqRenderedCommandRow]) -> Result<String> {
    let mut rendered = String::new();
    for row in rows {
        let payload = FastqRenderedCommandArgvRow {
            stage_id: row.stage_id.clone(),
            tool_id: row.tool_id.clone(),
            corpus_id: row.corpus_id.clone(),
            asset_profile_id: row.asset_profile_id.clone(),
            adapter_id: row.adapter_id.clone(),
            parser_id: row.parser_id.clone(),
            schema_id: row.schema_id.clone(),
            readiness_kind: row.readiness_kind.clone(),
            benchmark_status: row.benchmark_status.clone(),
            command_source: row.command_source.clone(),
            command_steps: row.command_steps.clone(),
        };
        let line =
            serde_json::to_string(&payload).context("serialize FASTQ rendered command argv row")?;
        rendered.push_str(&line);
        rendered.push('\n');
    }
    Ok(rendered)
}

fn binding_key(stage_id: &str, tool_id: &str) -> BindingKey {
    BindingKey { stage_id: stage_id.to_string(), tool_id: tool_id.to_string() }
}

fn companion_argv_output_path(output_path: &Path) -> PathBuf {
    let file_name = output_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(DEFAULT_FASTQ_RENDERED_COMMANDS_PATH);
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

fn repo_root_relative_to_output(repo_root: &Path, output_path: &Path) -> String {
    let relative_output_path = output_path.strip_prefix(repo_root).unwrap_or(output_path);
    let depth =
        relative_output_path.parent().map(|parent| parent.components().count()).unwrap_or(0);
    if depth == 0 {
        ".".to_string()
    } else {
        std::iter::repeat_n("..", depth).collect::<Vec<_>>().join("/")
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{render_fastq_commands, DEFAULT_FASTQ_RENDERED_COMMANDS_PATH};

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn fastq_rendered_commands_report_tracks_active_fastq_rows() {
        let root = repo_root();
        let report =
            render_fastq_commands(&root, PathBuf::from(DEFAULT_FASTQ_RENDERED_COMMANDS_PATH))
                .expect("render FASTQ commands");

        assert_eq!(report.schema_version, "bijux.bench.readiness.fastq_rendered_commands.v1");
        assert_eq!(report.output_path, "benchmarks/readiness/fastq/fastq-rendered-commands.sh");
        assert_eq!(
            report.argv_output_path,
            "benchmarks/readiness/fastq/fastq-rendered-commands.argv.jsonl"
        );
        assert_eq!(report.row_count, 69);
        assert_eq!(report.stage_count, 26);
        assert_eq!(report.tool_count, 41);
        assert_eq!(
            report.command_source_counts.get("fastq_bam_command_adapter"),
            Some(&69)
        );
        assert!(report.rows.iter().all(|row| {
            row.benchmark_status == "benchmark_ready"
                && row.command_source == "fastq_bam_command_adapter"
                && row.command_steps.len() == 1
                && row.command_steps[0].step_id == "invoke"
                && !row.command_steps[0].argv.is_empty()
                && row.script_commands.len() == 1
                && !row.script_commands[0].trim().is_empty()
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "fastq.cluster_otus"
                && row.tool_id == "vsearch"
                && row.command_steps[0].argv.first().map(String::as_str) == Some("vsearch")
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "fastq.screen_taxonomy"
                && row.tool_id == "kraken2"
                && row.asset_profile_id == "database_artifact_id+taxonomy_database_root"
                && row.command_steps[0].argv.first().map(String::as_str) == Some("sh")
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "fastq.infer_asvs"
                && row.tool_id == "dada2"
                && row.command_steps[0].argv.first().map(String::as_str) == Some("dada2")
        }));
    }
}
