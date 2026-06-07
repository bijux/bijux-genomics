use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Serialize;

use crate::commands::benchmark::local_stage_fake_runs::{
    fake_run_local_stage_commands, path_relative_to_repo, DEFAULT_LOCAL_STAGE_FAKE_RUN_ROOT,
};
use crate::commands::benchmark::local_stage_result_manifest::{
    load_validated_stage_result_manifest_path, BenchStageResultResourceMetricSource,
    BenchStageResultStatus,
};
use crate::commands::benchmark::local_stage_runtime_metrics::{
    collect_local_stage_runtime_metrics, DEFAULT_RUNTIME_METRICS_REPORT_PATH,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_TOOL_COMPARISON_TEMPLATE_PATH: &str =
    "benchmarks/readiness/local-ready/tool-comparison-template.tsv";
const LOCAL_TOOL_COMPARISON_TEMPLATE_SCHEMA_VERSION: &str =
    "bijux.bench.local_tool_comparison_template.v1";

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BenchLocalToolComparisonTemplateRow {
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) runtime_seconds: String,
    pub(crate) memory_mb: String,
    pub(crate) output_metric: String,
    pub(crate) status: String,
    pub(crate) failure_reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BenchLocalToolComparisonTemplateReport {
    pub(crate) schema_version: &'static str,
    pub(crate) fake_run_root: String,
    pub(crate) runtime_metrics_path: String,
    pub(crate) tsv_output_path: String,
    pub(crate) row_count: usize,
    pub(crate) rows: Vec<BenchLocalToolComparisonTemplateRow>,
}

pub(crate) fn run_render_tool_comparison_template(
    args: &parse::BenchLocalRenderToolComparisonTemplateArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_local_tool_comparison_template(
        &repo_root,
        args.fake_run_root
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_STAGE_FAKE_RUN_ROOT)),
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_TOOL_COMPARISON_TEMPLATE_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.tsv_output_path);
    }
    Ok(())
}

pub(crate) fn render_local_tool_comparison_template(
    repo_root: &Path,
    fake_run_root: PathBuf,
    output_path: PathBuf,
) -> Result<BenchLocalToolComparisonTemplateReport> {
    let absolute_fake_run_root =
        if fake_run_root.is_absolute() { fake_run_root } else { repo_root.join(&fake_run_root) };
    let absolute_output_path =
        if output_path.is_absolute() { output_path } else { repo_root.join(&output_path) };
    if let Some(parent) = absolute_output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    fake_run_local_stage_commands(repo_root, absolute_fake_run_root.clone())?;
    let runtime_metrics = collect_local_stage_runtime_metrics(
        repo_root,
        absolute_fake_run_root.clone(),
        PathBuf::from(DEFAULT_RUNTIME_METRICS_REPORT_PATH),
    )?;

    let rows = runtime_metrics
        .stages
        .iter()
        .map(|runtime_stage| {
            let manifest_path = repo_root.join(&runtime_stage.manifest_path);
            let manifest = load_validated_stage_result_manifest_path(&manifest_path)
                .with_context(|| format!("load {}", manifest_path.display()))?;
            Ok(BenchLocalToolComparisonTemplateRow {
                stage_id: runtime_stage.stage_id.clone(),
                tool_id: runtime_stage.tool_id.clone(),
                runtime_seconds: format!("{:.1}", runtime_stage.elapsed_seconds),
                memory_mb: memory_value(&manifest.resource_metrics),
                output_metric: "not_available".to_string(),
                status: status_label(&manifest.runtime.status),
                failure_reason: failure_reason_label(
                    &manifest.runtime.status,
                    manifest.runtime.exit_code,
                ),
            })
        })
        .collect::<Result<Vec<_>>>()?;

    let report = BenchLocalToolComparisonTemplateReport {
        schema_version: LOCAL_TOOL_COMPARISON_TEMPLATE_SCHEMA_VERSION,
        fake_run_root: path_relative_to_repo(repo_root, &absolute_fake_run_root),
        runtime_metrics_path: runtime_metrics.report_output_path,
        tsv_output_path: path_relative_to_repo(repo_root, &absolute_output_path),
        row_count: rows.len(),
        rows,
    };
    fs::write(&absolute_output_path, render_tool_comparison_template_tsv(&report.rows))
        .with_context(|| format!("write {}", absolute_output_path.display()))?;
    Ok(report)
}

fn memory_value(
    resource_metrics: &crate::commands::benchmark::local_stage_result_manifest::BenchStageResultResourceMetricsV1,
) -> String {
    match resource_metrics.source {
        BenchStageResultResourceMetricSource::Measured
        | BenchStageResultResourceMetricSource::Estimated => resource_metrics
            .memory_mb
            .map(|memory_mb| format!("{memory_mb:.1}"))
            .unwrap_or_else(|| "not_available".to_string()),
        BenchStageResultResourceMetricSource::NotAvailable => "not_available".to_string(),
    }
}

fn status_label(status: &BenchStageResultStatus) -> String {
    match status {
        BenchStageResultStatus::Succeeded => "succeeded".to_string(),
        BenchStageResultStatus::Failed => "failed".to_string(),
    }
}

fn failure_reason_label(status: &BenchStageResultStatus, exit_code: i32) -> String {
    match status {
        BenchStageResultStatus::Succeeded => "not_applicable".to_string(),
        BenchStageResultStatus::Failed => format!("exit_code_{exit_code}"),
    }
}

fn render_tool_comparison_template_tsv(rows: &[BenchLocalToolComparisonTemplateRow]) -> String {
    let mut rendered = String::from(
        "stage_id\ttool_id\truntime_seconds\tmemory_mb\toutput_metric\tstatus\tfailure_reason\n",
    );
    for row in rows {
        rendered.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            sanitize_tsv(&row.stage_id),
            sanitize_tsv(&row.tool_id),
            sanitize_tsv(&row.runtime_seconds),
            sanitize_tsv(&row.memory_mb),
            sanitize_tsv(&row.output_metric),
            sanitize_tsv(&row.status),
            sanitize_tsv(&row.failure_reason),
        ));
    }
    rendered
}

fn sanitize_tsv(value: &str) -> String {
    value.replace(['\t', '\n', '\r'], " ")
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    #[cfg(feature = "bam_downstream")]
    use super::{
        render_local_tool_comparison_template, DEFAULT_TOOL_COMPARISON_TEMPLATE_PATH,
        LOCAL_TOOL_COMPARISON_TEMPLATE_SCHEMA_VERSION,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[cfg(feature = "bam_downstream")]
    #[test]
    fn tool_comparison_template_reports_governed_51_stage_slice() {
        let root = repo_root();
        let report = render_local_tool_comparison_template(
            &root,
            PathBuf::from("target/local-fake-runs/stages-tool-comparison-template"),
            PathBuf::from(DEFAULT_TOOL_COMPARISON_TEMPLATE_PATH),
        )
        .expect("render tool comparison template");

        assert_eq!(report.schema_version, LOCAL_TOOL_COMPARISON_TEMPLATE_SCHEMA_VERSION);
        assert_eq!(report.row_count, 51);
        assert_eq!(report.rows.len(), 51);
        assert!(report.rows.iter().all(|row| {
            !row.stage_id.is_empty()
                && !row.tool_id.is_empty()
                && row.runtime_seconds == "1.0"
                && row.output_metric == "not_available"
                && row.status == "succeeded"
                && row.failure_reason == "not_applicable"
        }));
    }
}
