use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use crate::commands::benchmark::local_stage_fake_runs::{
    fake_run_local_stage_commands, DEFAULT_LOCAL_STAGE_FAKE_RUN_ROOT,
};
use crate::commands::benchmark::local_stage_inventory::LocalStageReadinessKind;
use crate::commands::benchmark::local_stage_manifest_completion::{
    check_local_stage_manifest_completion, BenchLocalStageManifestCompletionEntry,
};
use crate::commands::benchmark::local_stage_output_completion::{
    check_local_stage_output_completion, BenchLocalStageOutputCompletionEntry,
};
use crate::commands::benchmark::local_stage_result_manifest::BenchStageResultStatus;
use crate::commands::benchmark::local_stage_runtime_metrics::{
    collect_local_stage_runtime_metrics, BenchLocalStageRuntimeMetricEntry,
};
use crate::commands::benchmark::local_tool_comparison_template::{
    render_local_tool_comparison_template, BenchLocalToolComparisonTemplateRow,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

const DEFAULT_BENCHMARK_SUMMARY_JSON_PATH: &str =
    "benchmarks/readiness/local-ready/benchmark-summary.json";
const DEFAULT_BENCHMARK_SUMMARY_MARKDOWN_PATH: &str =
    "benchmarks/readiness/local-ready/benchmark-summary.md";
const DEFAULT_MANIFEST_COMPLETION_REPORT_PATH: &str =
    "benchmarks/readiness/local-ready/manifest-completion-report.json";
const DEFAULT_OUTPUT_COMPLETION_REPORT_PATH: &str =
    "benchmarks/readiness/local-ready/output-completion-report.json";
const DEFAULT_RUNTIME_METRICS_REPORT_PATH: &str =
    "benchmarks/readiness/local-ready/runtime-metrics.json";
const DEFAULT_TOOL_COMPARISON_TEMPLATE_PATH: &str =
    "benchmarks/readiness/local-ready/tool-comparison-template.tsv";
const LOCAL_BENCHMARK_SUMMARY_SCHEMA_VERSION: &str = "bijux.bench.local_benchmark_summary.v1";

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum BenchLocalBenchmarkReadinessStatus {
    Ready,
    MissingManifest,
    MissingOutputs,
    MissingManifestAndOutputs,
    Failed,
}

impl BenchLocalBenchmarkReadinessStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::MissingManifest => "missing_manifest",
            Self::MissingOutputs => "missing_outputs",
            Self::MissingManifestAndOutputs => "missing_manifest_and_outputs",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BenchLocalBenchmarkSummaryStage {
    pub(crate) stage_id: String,
    pub(crate) readiness_kind: LocalStageReadinessKind,
    pub(crate) tool_id: String,
    pub(crate) readiness_status: BenchLocalBenchmarkReadinessStatus,
    pub(crate) manifest_exists: bool,
    pub(crate) declared_output_count: usize,
    pub(crate) present_output_count: usize,
    pub(crate) missing_output_count: usize,
    pub(crate) runtime_seconds: String,
    pub(crate) runtime_status: String,
    pub(crate) memory_mb: String,
    pub(crate) output_metric: String,
    pub(crate) failure_reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BenchLocalBenchmarkSummaryReport {
    pub(crate) schema_version: &'static str,
    pub(crate) fake_run_root: String,
    pub(crate) report_output_path: String,
    pub(crate) markdown_output_path: String,
    pub(crate) source_stage_command_manifest_path: String,
    pub(crate) manifest_completion_report_path: String,
    pub(crate) output_completion_report_path: String,
    pub(crate) runtime_metrics_report_path: String,
    pub(crate) tool_comparison_template_path: String,
    pub(crate) stage_count: usize,
    pub(crate) ready_stage_count: usize,
    pub(crate) incomplete_stage_count: usize,
    pub(crate) failed_stage_count: usize,
    pub(crate) stages: Vec<BenchLocalBenchmarkSummaryStage>,
}

pub(crate) fn run_render_benchmark_summary(
    args: &parse::BenchLocalRenderBenchmarkSummaryArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_local_benchmark_summary(
        &repo_root,
        args.fake_run_root
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_STAGE_FAKE_RUN_ROOT)),
        args.output_json
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_BENCHMARK_SUMMARY_JSON_PATH)),
        args.output_markdown
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_BENCHMARK_SUMMARY_MARKDOWN_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.report_output_path);
    }
    Ok(())
}

pub(crate) fn render_local_benchmark_summary(
    repo_root: &Path,
    fake_run_root: PathBuf,
    json_output_path: PathBuf,
    markdown_output_path: PathBuf,
) -> Result<BenchLocalBenchmarkSummaryReport> {
    let absolute_fake_run_root =
        if fake_run_root.is_absolute() { fake_run_root } else { repo_root.join(&fake_run_root) };
    let absolute_json_output_path = if json_output_path.is_absolute() {
        json_output_path
    } else {
        repo_root.join(&json_output_path)
    };
    let absolute_markdown_output_path = if markdown_output_path.is_absolute() {
        markdown_output_path
    } else {
        repo_root.join(&markdown_output_path)
    };
    for path in [&absolute_json_output_path, &absolute_markdown_output_path] {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
        }
    }

    // Always refresh the fake-run tree so the benchmark summary cannot inherit
    // stale partial outputs from an earlier local check.
    fake_run_local_stage_commands(repo_root, absolute_fake_run_root.clone())?;
    // Render the comparison template before deriving completion and runtime rollups,
    // because that helper also refreshes the fake-run tree.
    let tool_comparison = render_local_tool_comparison_template(
        repo_root,
        absolute_fake_run_root.clone(),
        PathBuf::from(DEFAULT_TOOL_COMPARISON_TEMPLATE_PATH),
    )?;
    let manifest_completion = check_local_stage_manifest_completion(
        repo_root,
        absolute_fake_run_root.clone(),
        PathBuf::from(DEFAULT_MANIFEST_COMPLETION_REPORT_PATH),
    )?;
    let output_completion = check_local_stage_output_completion(
        repo_root,
        absolute_fake_run_root.clone(),
        PathBuf::from(DEFAULT_OUTPUT_COMPLETION_REPORT_PATH),
    )?;
    let runtime_metrics = collect_local_stage_runtime_metrics(
        repo_root,
        absolute_fake_run_root.clone(),
        PathBuf::from(DEFAULT_RUNTIME_METRICS_REPORT_PATH),
    )?;

    let output_by_stage = output_completion
        .stages
        .iter()
        .map(|stage| (stage.stage_id.as_str(), stage))
        .collect::<BTreeMap<_, _>>();
    let runtime_by_stage = runtime_metrics
        .stages
        .iter()
        .map(|stage| (stage.stage_id.as_str(), stage))
        .collect::<BTreeMap<_, _>>();
    let comparison_by_stage = tool_comparison
        .rows
        .iter()
        .map(|row| (row.stage_id.as_str(), row))
        .collect::<BTreeMap<_, _>>();

    let stages = manifest_completion
        .stages
        .iter()
        .map(|manifest_stage| {
            let output_stage =
                output_by_stage.get(manifest_stage.stage_id.as_str()).copied().ok_or_else(
                    || missing_stage_error("output completion", &manifest_stage.stage_id),
                )?;
            let runtime_stage = runtime_by_stage
                .get(manifest_stage.stage_id.as_str())
                .copied()
                .ok_or_else(|| missing_stage_error("runtime metrics", &manifest_stage.stage_id))?;
            let comparison_stage =
                comparison_by_stage.get(manifest_stage.stage_id.as_str()).copied().ok_or_else(
                    || missing_stage_error("tool comparison template", &manifest_stage.stage_id),
                )?;
            Ok(build_summary_stage(manifest_stage, output_stage, runtime_stage, comparison_stage))
        })
        .collect::<Result<Vec<_>>>()?;

    let ready_stage_count = stages
        .iter()
        .filter(|stage| stage.readiness_status == BenchLocalBenchmarkReadinessStatus::Ready)
        .count();
    let failed_stage_count = stages
        .iter()
        .filter(|stage| stage.readiness_status == BenchLocalBenchmarkReadinessStatus::Failed)
        .count();
    let incomplete_stage_count = stages
        .iter()
        .filter(|stage| stage.readiness_status != BenchLocalBenchmarkReadinessStatus::Ready)
        .count();

    let report = BenchLocalBenchmarkSummaryReport {
        schema_version: LOCAL_BENCHMARK_SUMMARY_SCHEMA_VERSION,
        fake_run_root: manifest_completion.fake_run_root,
        report_output_path: path_relative_to_repo(repo_root, &absolute_json_output_path),
        markdown_output_path: path_relative_to_repo(repo_root, &absolute_markdown_output_path),
        source_stage_command_manifest_path: manifest_completion.source_stage_command_manifest_path,
        manifest_completion_report_path: manifest_completion.report_output_path,
        output_completion_report_path: output_completion.report_output_path,
        runtime_metrics_report_path: runtime_metrics.report_output_path,
        tool_comparison_template_path: tool_comparison.tsv_output_path,
        stage_count: stages.len(),
        ready_stage_count,
        incomplete_stage_count,
        failed_stage_count,
        stages,
    };

    bijux_dna_infra::atomic_write_json(&absolute_json_output_path, &report)?;
    fs::write(&absolute_markdown_output_path, render_local_benchmark_summary_markdown(&report))
        .with_context(|| format!("write {}", absolute_markdown_output_path.display()))?;

    Ok(report)
}

fn build_summary_stage(
    manifest_stage: &BenchLocalStageManifestCompletionEntry,
    output_stage: &BenchLocalStageOutputCompletionEntry,
    runtime_stage: &BenchLocalStageRuntimeMetricEntry,
    comparison_stage: &BenchLocalToolComparisonTemplateRow,
) -> BenchLocalBenchmarkSummaryStage {
    BenchLocalBenchmarkSummaryStage {
        stage_id: manifest_stage.stage_id.clone(),
        readiness_kind: manifest_stage.readiness_kind,
        tool_id: manifest_stage.tool_id.clone(),
        readiness_status: summary_status(manifest_stage, output_stage, runtime_stage),
        manifest_exists: manifest_stage.manifest_exists,
        declared_output_count: output_stage.declared_output_count,
        present_output_count: output_stage.present_output_count,
        missing_output_count: output_stage.missing_output_count,
        runtime_seconds: comparison_stage.runtime_seconds.clone(),
        runtime_status: runtime_status_label(&runtime_stage.status).to_string(),
        memory_mb: comparison_stage.memory_mb.clone(),
        output_metric: comparison_stage.output_metric.clone(),
        failure_reason: comparison_stage.failure_reason.clone(),
    }
}

fn summary_status(
    manifest_stage: &BenchLocalStageManifestCompletionEntry,
    output_stage: &BenchLocalStageOutputCompletionEntry,
    runtime_stage: &BenchLocalStageRuntimeMetricEntry,
) -> BenchLocalBenchmarkReadinessStatus {
    let missing_manifest = !manifest_stage.manifest_exists;
    let missing_outputs = output_stage.missing_output_count > 0;
    match (missing_manifest, missing_outputs, &runtime_stage.status) {
        (true, true, _) => BenchLocalBenchmarkReadinessStatus::MissingManifestAndOutputs,
        (true, false, _) => BenchLocalBenchmarkReadinessStatus::MissingManifest,
        (false, true, _) => BenchLocalBenchmarkReadinessStatus::MissingOutputs,
        (false, false, BenchStageResultStatus::Failed) => {
            BenchLocalBenchmarkReadinessStatus::Failed
        }
        (false, false, BenchStageResultStatus::Succeeded) => {
            BenchLocalBenchmarkReadinessStatus::Ready
        }
    }
}

fn runtime_status_label(status: &BenchStageResultStatus) -> &'static str {
    match status {
        BenchStageResultStatus::Succeeded => "succeeded",
        BenchStageResultStatus::Failed => "failed",
    }
}

fn render_local_benchmark_summary_markdown(report: &BenchLocalBenchmarkSummaryReport) -> String {
    let mut rendered = String::from("# Local Benchmark Summary\n\n");
    rendered.push_str(&format!(
        "- Stage count: `{}`\n- Ready stages: `{}`\n- Incomplete stages: `{}`\n- Failed stages: `{}`\n\n",
        report.stage_count,
        report.ready_stage_count,
        report.incomplete_stage_count,
        report.failed_stage_count,
    ));
    rendered.push_str("## Sources\n\n");
    rendered.push_str(&format!(
        "- Fake-run root: `{}`\n- Stage command manifest: `{}`\n- Manifest completion report: `{}`\n- Output completion report: `{}`\n- Runtime metrics report: `{}`\n- Tool comparison template: `{}`\n\n",
        report.fake_run_root,
        report.source_stage_command_manifest_path,
        report.manifest_completion_report_path,
        report.output_completion_report_path,
        report.runtime_metrics_report_path,
        report.tool_comparison_template_path,
    ));
    rendered.push_str("## Stage Readiness\n\n");
    rendered.push_str(
        "| Stage | Tool | Readiness Kind | Readiness Status | Runtime (s) | Memory (MB) | Failure Reason |\n",
    );
    rendered.push_str("| --- | --- | --- | --- | ---: | ---: | --- |\n");
    for stage in &report.stages {
        rendered.push_str(&format!(
            "| `{}` | `{}` | `{}` | `{}` | `{}` | `{}` | `{}` |\n",
            sanitize_markdown(&stage.stage_id),
            sanitize_markdown(&stage.tool_id),
            stage.readiness_kind.as_str(),
            stage.readiness_status.as_str(),
            sanitize_markdown(&stage.runtime_seconds),
            sanitize_markdown(&stage.memory_mb),
            sanitize_markdown(&stage.failure_reason),
        ));
    }
    rendered
}

fn sanitize_markdown(value: &str) -> String {
    value.replace('|', "\\|").replace(['\n', '\r'], " ")
}

fn missing_stage_error(report_name: &str, stage_id: &str) -> anyhow::Error {
    anyhow!("{report_name} does not contain governed stage `{stage_id}`")
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).display().to_string()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    #[cfg(feature = "bam_downstream")]
    use super::{
        render_local_benchmark_summary, BenchLocalBenchmarkReadinessStatus,
        DEFAULT_BENCHMARK_SUMMARY_JSON_PATH, DEFAULT_BENCHMARK_SUMMARY_MARKDOWN_PATH,
        LOCAL_BENCHMARK_SUMMARY_SCHEMA_VERSION,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[cfg(feature = "bam_downstream")]
    #[test]
    fn benchmark_summary_reports_governed_51_stage_slice() {
        let root = repo_root();
        let report = render_local_benchmark_summary(
            &root,
            PathBuf::from("runs/bench/local-fake-runs/stages-benchmark-summary"),
            PathBuf::from(DEFAULT_BENCHMARK_SUMMARY_JSON_PATH),
            PathBuf::from(DEFAULT_BENCHMARK_SUMMARY_MARKDOWN_PATH),
        )
        .expect("render local benchmark summary");

        assert_eq!(report.schema_version, LOCAL_BENCHMARK_SUMMARY_SCHEMA_VERSION);
        assert_eq!(report.stage_count, 51);
        assert_eq!(report.ready_stage_count, 51);
        assert_eq!(report.incomplete_stage_count, 0);
        assert_eq!(report.failed_stage_count, 0);
        assert!(report.stages.iter().all(|stage| {
            stage.readiness_status == BenchLocalBenchmarkReadinessStatus::Ready
                && stage.runtime_status == "succeeded"
                && stage.manifest_exists
                && stage.missing_output_count == 0
        }));
    }
}
