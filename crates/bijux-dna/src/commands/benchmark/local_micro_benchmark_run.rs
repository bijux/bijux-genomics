use std::fmt::Write as _;
use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, bail, Context, Result};
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use super::local_adna_micro_pipeline::{
    render_adna_micro_pipeline, AdnaMicroPipelineReport, AdnaMicroPipelineRowStatus,
};
use super::local_bam_micro_smoke_subset::{
    render_bam_micro_smoke_subset, BamMicroSmokeExecutionStatus, BamMicroSmokeSubsetReport,
};
use super::local_core_germline_micro_pipeline::{
    render_core_germline_micro_pipeline, CoreGermlineMicroPipelineReport,
};
use super::local_fastq_micro_smoke_subset::{
    render_fastq_micro_smoke_subset, FastqMicroSmokeExecutionStatus, FastqMicroSmokeSubsetReport,
};
use super::local_real_smoke_core_subset::{
    render_real_smoke_core_subset, RealSmokeCoreSubsetExecutionKind, RealSmokeCoreSubsetReport,
};
use super::local_stage_result_manifest::{
    load_validated_stage_result_manifest_path, path_relative_to_repo, BenchStageResultManifestV1,
};
use super::local_vcf_micro_smoke_subset::{
    render_vcf_micro_smoke_subset, VcfMicroSmokeExecutionStatus, VcfMicroSmokeSubsetReport,
};
use super::path_resolution::{
    ensure_path_stays_outside_benchmark_readiness_root,
    ensure_path_stays_within_benchmark_runs_root, BenchmarkPathResolver,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_MICRO_BENCHMARK_RUN_MANIFEST_PATH: &str =
    "runs/bench/micro/MICRO_BENCHMARK_RUN.json";
const MICRO_BENCHMARK_RUN_SCHEMA_VERSION: &str = "bijux.bench.local_micro_benchmark_run.v1";
const MICRO_BENCHMARK_RUN_COMMAND: &str = "bijux-dna bench run-micro";
const MICRO_BENCHMARK_RESULT_ROWS_NAME: &str = "MICRO_RESULT_ROWS.json";
const MICRO_BENCHMARK_OUTPUT_ROWS_NAME: &str = "MICRO_OUTPUT_ROWS.json";
const MICRO_BENCHMARK_LOG_ROWS_NAME: &str = "MICRO_LOG_ROWS.json";
const MICRO_BENCHMARK_NORMALIZED_METRICS_NAME: &str = "MICRO_NORMALIZED_METRICS.json";
const MICRO_BENCHMARK_LOG_NAME: &str = "MICRO_RUN.log";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum MicroBenchmarkResultKind {
    Stage,
    PipelineBridge,
    FamilyRepresentative,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum MicroBenchmarkExecutionStatus {
    Succeeded,
    ContainerNeeded,
    Unavailable,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct MicroBenchmarkComponentReport {
    pub(crate) component_id: String,
    pub(crate) report_path: String,
    pub(crate) row_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct MicroBenchmarkResultRow {
    pub(crate) execution_id: String,
    pub(crate) component_id: String,
    pub(crate) result_kind: MicroBenchmarkResultKind,
    pub(crate) domain: String,
    pub(crate) bridge_source_domain: Option<String>,
    pub(crate) bridge_target_domain: Option<String>,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) status: MicroBenchmarkExecutionStatus,
    pub(crate) reason: String,
    pub(crate) command: Option<String>,
    pub(crate) source_report_path: String,
    pub(crate) evidence_path: Option<String>,
    pub(crate) stage_result_manifest_path: Option<String>,
    pub(crate) normalized_metric_count: usize,
    pub(crate) output_count: usize,
    pub(crate) log_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct MicroBenchmarkOutputRow {
    pub(crate) execution_id: String,
    pub(crate) component_id: String,
    pub(crate) artifact_id: String,
    pub(crate) role: String,
    pub(crate) path: String,
    pub(crate) exists: bool,
    pub(crate) source: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct MicroBenchmarkLogRow {
    pub(crate) execution_id: String,
    pub(crate) component_id: String,
    pub(crate) role: String,
    pub(crate) path: String,
    pub(crate) exists: bool,
    pub(crate) source: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct MicroBenchmarkNormalizedMetricRow {
    pub(crate) execution_id: String,
    pub(crate) component_id: String,
    pub(crate) metric_id: String,
    pub(crate) value: Value,
    pub(crate) source_path: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct MicroBenchmarkRunManifest {
    pub(crate) schema_version: &'static str,
    pub(crate) manifest_path: String,
    pub(crate) run_root: String,
    pub(crate) run_id: String,
    pub(crate) repo_revision: String,
    pub(crate) worktree_dirty: bool,
    pub(crate) created_at_unix: u64,
    pub(crate) command: &'static str,
    pub(crate) component_reports: Vec<MicroBenchmarkComponentReport>,
    pub(crate) result_rows_path: String,
    pub(crate) output_rows_path: String,
    pub(crate) log_rows_path: String,
    pub(crate) normalized_metrics_path: String,
    pub(crate) result_row_count: usize,
    pub(crate) output_row_count: usize,
    pub(crate) log_row_count: usize,
    pub(crate) normalized_metric_row_count: usize,
    pub(crate) passes_behavior_test: bool,
}

pub(crate) fn run_micro_benchmark(args: &parse::BenchRunMicroArgs) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_micro_benchmark_run(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_MICRO_BENCHMARK_RUN_MANIFEST_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.manifest_path);
    }
    Ok(())
}

pub(crate) fn render_micro_benchmark_run(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<MicroBenchmarkRunManifest> {
    let benchmark_paths = BenchmarkPathResolver::new(repo_root, None);
    let manifest_output_path = benchmark_paths.resolve_repo_relative(&output_path);
    let run_root = benchmark_paths.benchmark_micro_root();
    let results_path = run_root.join("results").join(MICRO_BENCHMARK_RESULT_ROWS_NAME);
    let outputs_path = run_root.join("outputs").join(MICRO_BENCHMARK_OUTPUT_ROWS_NAME);
    let log_rows_path = run_root.join("logs").join(MICRO_BENCHMARK_LOG_ROWS_NAME);
    let log_path = run_root.join("logs").join(MICRO_BENCHMARK_LOG_NAME);
    let normalized_metrics_path =
        run_root.join("normalized-metrics").join(MICRO_BENCHMARK_NORMALIZED_METRICS_NAME);
    let core_report_path = run_root.join("core").join("REAL_SMOKE_CORE_SUMMARY.json");
    let fastq_report_path = run_root.join("fastq").join("MICRO_FASTQ_SUMMARY.json");
    let bam_report_path = run_root.join("bam").join("MICRO_BAM_SUMMARY.json");
    let vcf_report_path = run_root.join("vcf").join("MICRO_VCF_SUMMARY.json");
    let adna_report_path = run_root.join("pipelines").join("adna").join("MICRO_ADNA_SUMMARY.json");
    let pipeline_report_path =
        run_root.join("pipelines").join("core-germline").join("MICRO_PIPELINE_SUMMARY.json");

    ensure_path_stays_outside_benchmark_readiness_root(
        repo_root,
        &manifest_output_path,
        "micro benchmark manifest output",
    )?;
    ensure_path_stays_within_benchmark_runs_root(repo_root, &run_root, "micro benchmark run root")?;

    for directory in [
        run_root.as_path(),
        results_path
            .parent()
            .ok_or_else(|| anyhow!("micro benchmark results parent is missing"))?,
        outputs_path
            .parent()
            .ok_or_else(|| anyhow!("micro benchmark outputs parent is missing"))?,
        log_rows_path.parent().ok_or_else(|| anyhow!("micro benchmark logs parent is missing"))?,
        normalized_metrics_path
            .parent()
            .ok_or_else(|| anyhow!("micro benchmark metrics parent is missing"))?,
        core_report_path
            .parent()
            .ok_or_else(|| anyhow!("micro benchmark core parent is missing"))?,
        fastq_report_path
            .parent()
            .ok_or_else(|| anyhow!("micro benchmark FASTQ parent is missing"))?,
        bam_report_path.parent().ok_or_else(|| anyhow!("micro benchmark BAM parent is missing"))?,
        vcf_report_path.parent().ok_or_else(|| anyhow!("micro benchmark VCF parent is missing"))?,
        adna_report_path
            .parent()
            .ok_or_else(|| anyhow!("micro benchmark aDNA pipeline parent is missing"))?,
        pipeline_report_path
            .parent()
            .ok_or_else(|| anyhow!("micro benchmark pipeline parent is missing"))?,
    ] {
        fs::create_dir_all(directory).with_context(|| format!("create {}", directory.display()))?;
    }
    if let Some(parent) = manifest_output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    let repo_revision = git_stdout(repo_root, &["rev-parse", "HEAD"])?;
    let worktree_dirty =
        !git_stdout(repo_root, &["status", "--short", "--untracked-files=no"])?.trim().is_empty();
    let created_at_unix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock drifted before unix epoch")?
        .as_secs();

    let mut run_log = Vec::<String>::new();
    run_log.push(format!("command={MICRO_BENCHMARK_RUN_COMMAND}"));
    run_log.push(format!("repo_revision={repo_revision}"));
    run_log.push(format!("worktree_dirty={worktree_dirty}"));

    let real_report = render_real_smoke_core_subset(repo_root, core_report_path.clone())
        .context("render micro benchmark real-smoke core report")?;
    run_log.push(format!("real_smoke_core_subset={}", real_report.output_path));

    let fastq_report = render_fastq_micro_smoke_subset(repo_root, fastq_report_path.clone())
        .context("render micro benchmark FASTQ family report")?;
    run_log.push(format!("fastq_micro_smoke_subset={}", fastq_report.output_path));

    let bam_report = render_bam_micro_smoke_subset(repo_root, bam_report_path.clone())
        .context("render micro benchmark BAM family report")?;
    run_log.push(format!("bam_micro_smoke_subset={}", bam_report.output_path));

    let vcf_report = render_vcf_micro_smoke_subset(repo_root, vcf_report_path.clone())
        .context("render micro benchmark VCF family report")?;
    run_log.push(format!("vcf_micro_smoke_subset={}", vcf_report.output_path));

    let adna_report = render_adna_micro_pipeline(repo_root, adna_report_path.clone())
        .context("render micro benchmark aDNA pipeline report")?;
    run_log.push(format!("adna_micro_pipeline={}", adna_report.output_path));

    let pipeline_report = render_core_germline_micro_pipeline(repo_root, pipeline_report_path)
        .context("render micro benchmark core germline pipeline report")?;
    run_log.push(format!("core_germline_micro_pipeline={}", pipeline_report.output_path));

    let mut result_rows = Vec::new();
    let mut output_rows = Vec::new();
    let mut log_rows = Vec::new();
    let mut normalized_metric_rows = Vec::new();

    collect_real_smoke_rows(
        repo_root,
        &real_report,
        &mut result_rows,
        &mut output_rows,
        &mut log_rows,
        &mut normalized_metric_rows,
    )?;
    collect_fastq_micro_rows(repo_root, &fastq_report, &mut result_rows, &mut output_rows)?;
    collect_bam_micro_rows(
        repo_root,
        &bam_report,
        &mut result_rows,
        &mut output_rows,
        &mut log_rows,
    )?;
    collect_vcf_micro_rows(repo_root, &vcf_report, &mut result_rows, &mut output_rows)?;
    collect_adna_pipeline_rows(repo_root, &adna_report, &mut result_rows, &mut output_rows)?;
    collect_core_germline_pipeline_rows(
        repo_root,
        &pipeline_report,
        &mut result_rows,
        &mut output_rows,
    )?;

    result_rows.sort_by(|left, right| {
        left.component_id
            .cmp(&right.component_id)
            .then_with(|| left.execution_id.cmp(&right.execution_id))
            .then_with(|| left.stage_id.cmp(&right.stage_id))
            .then_with(|| left.tool_id.cmp(&right.tool_id))
    });
    output_rows.sort_by(|left, right| {
        left.execution_id
            .cmp(&right.execution_id)
            .then_with(|| left.artifact_id.cmp(&right.artifact_id))
            .then_with(|| left.path.cmp(&right.path))
    });
    log_rows.sort_by(|left, right| {
        left.execution_id.cmp(&right.execution_id).then_with(|| left.path.cmp(&right.path))
    });
    normalized_metric_rows.sort_by(|left, right| {
        left.execution_id
            .cmp(&right.execution_id)
            .then_with(|| left.metric_id.cmp(&right.metric_id))
    });

    run_log.push(format!("result_row_count={}", result_rows.len()));
    run_log.push(format!("output_row_count={}", output_rows.len()));
    run_log.push(format!("log_row_count={}", log_rows.len() + 1));
    run_log.push(format!("normalized_metric_row_count={}", normalized_metric_rows.len()));
    write_run_log(&log_path, &run_log)?;
    log_rows.push(MicroBenchmarkLogRow {
        execution_id: "micro.run".to_string(),
        component_id: "micro_benchmark_run".to_string(),
        role: "run_log".to_string(),
        path: path_relative_to_repo(repo_root, &log_path),
        exists: true,
        source: "micro_benchmark_run".to_string(),
    });
    log_rows.sort_by(|left, right| {
        left.execution_id.cmp(&right.execution_id).then_with(|| left.path.cmp(&right.path))
    });

    bijux_dna_infra::atomic_write_json(&results_path, &result_rows)?;
    bijux_dna_infra::atomic_write_json(&outputs_path, &output_rows)?;
    bijux_dna_infra::atomic_write_json(&log_rows_path, &log_rows)?;
    bijux_dna_infra::atomic_write_json(&normalized_metrics_path, &normalized_metric_rows)?;

    let mut manifest = MicroBenchmarkRunManifest {
        schema_version: MICRO_BENCHMARK_RUN_SCHEMA_VERSION,
        manifest_path: path_relative_to_repo(repo_root, &manifest_output_path),
        run_root: path_relative_to_repo(repo_root, &run_root),
        run_id: build_run_id(
            &repo_revision,
            &result_rows,
            &output_rows,
            &log_rows,
            &normalized_metric_rows,
        ),
        repo_revision,
        worktree_dirty,
        created_at_unix,
        command: MICRO_BENCHMARK_RUN_COMMAND,
        component_reports: vec![
            MicroBenchmarkComponentReport {
                component_id: "real_smoke_core_subset".to_string(),
                report_path: real_report.output_path.clone(),
                row_count: real_report.rows.len(),
            },
            MicroBenchmarkComponentReport {
                component_id: "fastq_micro_smoke_subset".to_string(),
                report_path: fastq_report.output_path.clone(),
                row_count: fastq_report.rows.len(),
            },
            MicroBenchmarkComponentReport {
                component_id: "bam_micro_smoke_subset".to_string(),
                report_path: bam_report.output_path.clone(),
                row_count: bam_report.rows.len(),
            },
            MicroBenchmarkComponentReport {
                component_id: "vcf_micro_smoke_subset".to_string(),
                report_path: vcf_report.output_path.clone(),
                row_count: vcf_report.rows.len(),
            },
            MicroBenchmarkComponentReport {
                component_id: "adna_micro_pipeline".to_string(),
                report_path: adna_report.output_path.clone(),
                row_count: adna_report.rows.len(),
            },
            MicroBenchmarkComponentReport {
                component_id: "core_germline_micro_pipeline".to_string(),
                report_path: pipeline_report.output_path.clone(),
                row_count: pipeline_report.rows.len(),
            },
        ],
        result_rows_path: path_relative_to_repo(repo_root, &results_path),
        output_rows_path: path_relative_to_repo(repo_root, &outputs_path),
        log_rows_path: path_relative_to_repo(repo_root, &log_rows_path),
        normalized_metrics_path: path_relative_to_repo(repo_root, &normalized_metrics_path),
        result_row_count: result_rows.len(),
        output_row_count: output_rows.len(),
        log_row_count: log_rows.len(),
        normalized_metric_row_count: normalized_metric_rows.len(),
        passes_behavior_test: false,
    };
    ensure_micro_benchmark_run_contract(
        repo_root,
        &mut manifest,
        &result_rows,
        &output_rows,
        &log_rows,
        &normalized_metric_rows,
    )?;
    bijux_dna_infra::atomic_write_json(&manifest_output_path, &manifest)?;
    Ok(manifest)
}

fn collect_real_smoke_rows(
    repo_root: &Path,
    report: &RealSmokeCoreSubsetReport,
    result_rows: &mut Vec<MicroBenchmarkResultRow>,
    output_rows: &mut Vec<MicroBenchmarkOutputRow>,
    log_rows: &mut Vec<MicroBenchmarkLogRow>,
    normalized_metric_rows: &mut Vec<MicroBenchmarkNormalizedMetricRow>,
) -> Result<()> {
    for row in &report.rows {
        let mut command = None;
        let mut manifest_outputs = 0usize;
        let mut manifest_logs = 0usize;
        if let Some(manifest_path) = &row.stage_result_manifest_path {
            let manifest_abs = repo_root.join(manifest_path);
            let manifest = load_validated_stage_result_manifest_path(&manifest_abs)
                .with_context(|| format!("load {}", manifest_abs.display()))?;
            command = Some(manifest.command.rendered.clone());
            let (output_count, log_count) =
                append_manifest_outputs(repo_root, row, &manifest, output_rows, log_rows);
            manifest_outputs = output_count;
            manifest_logs = log_count;
        }

        output_rows.push(MicroBenchmarkOutputRow {
            execution_id: row.execution_id.clone(),
            component_id: "real_smoke_core_subset".to_string(),
            artifact_id: "evidence".to_string(),
            role: "evidence_report".to_string(),
            path: row.evidence_path.clone(),
            exists: repo_root.join(&row.evidence_path).is_file(),
            source: "real_smoke_core_subset".to_string(),
        });

        for (metric_id, value) in &row.normalized_metrics {
            normalized_metric_rows.push(MicroBenchmarkNormalizedMetricRow {
                execution_id: row.execution_id.clone(),
                component_id: "real_smoke_core_subset".to_string(),
                metric_id: metric_id.clone(),
                value: value.clone(),
                source_path: row.evidence_path.clone(),
            });
        }

        result_rows.push(MicroBenchmarkResultRow {
            execution_id: row.execution_id.clone(),
            component_id: "real_smoke_core_subset".to_string(),
            result_kind: real_result_kind(row.execution_kind),
            domain: row.domain.clone(),
            bridge_source_domain: row.bridge_source_domain.clone(),
            bridge_target_domain: row.bridge_target_domain.clone(),
            stage_id: row.stage_id.clone(),
            tool_id: row.tool_id.clone(),
            status: MicroBenchmarkExecutionStatus::Succeeded,
            reason: "real_smoke_execution".to_string(),
            command,
            source_report_path: report.output_path.clone(),
            evidence_path: Some(row.evidence_path.clone()),
            stage_result_manifest_path: row.stage_result_manifest_path.clone(),
            normalized_metric_count: row.normalized_metric_count,
            output_count: manifest_outputs + 1,
            log_count: manifest_logs,
        });
    }
    Ok(())
}

fn collect_bam_micro_rows(
    repo_root: &Path,
    report: &BamMicroSmokeSubsetReport,
    result_rows: &mut Vec<MicroBenchmarkResultRow>,
    output_rows: &mut Vec<MicroBenchmarkOutputRow>,
    _log_rows: &mut Vec<MicroBenchmarkLogRow>,
) -> Result<()> {
    for row in &report.rows {
        let output_count = if let Some(evidence_path) = &row.evidence_path {
            output_rows.push(MicroBenchmarkOutputRow {
                execution_id: row.family_id.clone(),
                component_id: "bam_micro_smoke_subset".to_string(),
                artifact_id: "evidence".to_string(),
                role: "family_evidence".to_string(),
                path: evidence_path.clone(),
                exists: repo_root.join(evidence_path).is_file(),
                source: "bam_micro_smoke_subset".to_string(),
            });
            1
        } else {
            0
        };

        result_rows.push(MicroBenchmarkResultRow {
            execution_id: row.family_id.clone(),
            component_id: "bam_micro_smoke_subset".to_string(),
            result_kind: MicroBenchmarkResultKind::FamilyRepresentative,
            domain: "bam".to_string(),
            bridge_source_domain: None,
            bridge_target_domain: None,
            stage_id: row.representative_stage_id.clone(),
            tool_id: row.representative_tool_id.clone(),
            status: bam_result_status(row.execution_status.clone()),
            reason: row.reason.clone(),
            command: Some(row.smoke_command.clone()),
            source_report_path: report.output_path.clone(),
            evidence_path: row.evidence_path.clone(),
            stage_result_manifest_path: None,
            normalized_metric_count: 0,
            output_count,
            log_count: 0,
        });
    }
    Ok(())
}

fn collect_vcf_micro_rows(
    repo_root: &Path,
    report: &VcfMicroSmokeSubsetReport,
    result_rows: &mut Vec<MicroBenchmarkResultRow>,
    output_rows: &mut Vec<MicroBenchmarkOutputRow>,
) -> Result<()> {
    for row in &report.rows {
        let output_count = if let Some(evidence_path) = &row.evidence_path {
            output_rows.push(MicroBenchmarkOutputRow {
                execution_id: row.family_id.clone(),
                component_id: "vcf_micro_smoke_subset".to_string(),
                artifact_id: "evidence".to_string(),
                role: "family_evidence".to_string(),
                path: evidence_path.clone(),
                exists: repo_root.join(evidence_path).is_file(),
                source: "vcf_micro_smoke_subset".to_string(),
            });
            1
        } else {
            0
        };

        result_rows.push(MicroBenchmarkResultRow {
            execution_id: row.family_id.clone(),
            component_id: "vcf_micro_smoke_subset".to_string(),
            result_kind: MicroBenchmarkResultKind::FamilyRepresentative,
            domain: "vcf".to_string(),
            bridge_source_domain: None,
            bridge_target_domain: None,
            stage_id: row.representative_stage_id.clone(),
            tool_id: row.representative_tool_id.clone(),
            status: vcf_result_status(row.execution_status),
            reason: row.reason.clone(),
            command: Some(row.smoke_command.clone()),
            source_report_path: report.output_path.clone(),
            evidence_path: row.evidence_path.clone(),
            stage_result_manifest_path: None,
            normalized_metric_count: 0,
            output_count,
            log_count: 0,
        });
    }
    Ok(())
}

fn collect_fastq_micro_rows(
    repo_root: &Path,
    report: &FastqMicroSmokeSubsetReport,
    result_rows: &mut Vec<MicroBenchmarkResultRow>,
    output_rows: &mut Vec<MicroBenchmarkOutputRow>,
) -> Result<()> {
    for row in &report.rows {
        let output_count = if let Some(evidence_path) = &row.evidence_path {
            output_rows.push(MicroBenchmarkOutputRow {
                execution_id: row.family_id.clone(),
                component_id: "fastq_micro_smoke_subset".to_string(),
                artifact_id: "evidence".to_string(),
                role: "family_evidence".to_string(),
                path: evidence_path.clone(),
                exists: repo_root.join(evidence_path).is_file(),
                source: "fastq_micro_smoke_subset".to_string(),
            });
            1
        } else {
            0
        };

        result_rows.push(MicroBenchmarkResultRow {
            execution_id: row.family_id.clone(),
            component_id: "fastq_micro_smoke_subset".to_string(),
            result_kind: MicroBenchmarkResultKind::FamilyRepresentative,
            domain: "fastq".to_string(),
            bridge_source_domain: None,
            bridge_target_domain: None,
            stage_id: row.representative_stage_id.clone(),
            tool_id: row.representative_tool_id.clone(),
            status: fastq_result_status(row.execution_status),
            reason: row.reason.clone(),
            command: Some(row.smoke_command.clone()),
            source_report_path: report.output_path.clone(),
            evidence_path: row.evidence_path.clone(),
            stage_result_manifest_path: None,
            normalized_metric_count: 0,
            output_count,
            log_count: 0,
        });
    }
    Ok(())
}

fn collect_adna_pipeline_rows(
    repo_root: &Path,
    report: &AdnaMicroPipelineReport,
    result_rows: &mut Vec<MicroBenchmarkResultRow>,
    output_rows: &mut Vec<MicroBenchmarkOutputRow>,
) -> Result<()> {
    for row in &report.rows {
        if row.status == AdnaMicroPipelineRowStatus::Succeeded {
            if let Some(evidence_path) = &row.evidence_path {
                output_rows.push(MicroBenchmarkOutputRow {
                    execution_id: row.stage_id.clone(),
                    component_id: "adna_micro_pipeline".to_string(),
                    artifact_id: "evidence".to_string(),
                    role: "pipeline_stage_evidence".to_string(),
                    path: evidence_path.clone(),
                    exists: repo_root.join(evidence_path).is_file(),
                    source: "adna_micro_pipeline".to_string(),
                });
            }

            for (artifact_id, path) in &row.outputs {
                output_rows.push(MicroBenchmarkOutputRow {
                    execution_id: row.stage_id.clone(),
                    component_id: "adna_micro_pipeline".to_string(),
                    artifact_id: artifact_id.clone(),
                    role: "pipeline_stage_output".to_string(),
                    path: path.clone(),
                    exists: repo_root.join(path).exists(),
                    source: "adna_micro_pipeline".to_string(),
                });
            }
        }

        result_rows.push(MicroBenchmarkResultRow {
            execution_id: row.stage_id.clone(),
            component_id: "adna_micro_pipeline".to_string(),
            result_kind: if row.domain == "vcf" {
                MicroBenchmarkResultKind::PipelineBridge
            } else {
                MicroBenchmarkResultKind::Stage
            },
            domain: row.domain.clone(),
            bridge_source_domain: None,
            bridge_target_domain: None,
            stage_id: row.stage_id.clone(),
            tool_id: row.tool_id.clone(),
            status: if row.status == AdnaMicroPipelineRowStatus::Succeeded {
                MicroBenchmarkExecutionStatus::Succeeded
            } else {
                MicroBenchmarkExecutionStatus::Unavailable
            },
            reason: row.reason.clone(),
            command: Some("bijux-dna bench local run-adna-micro-pipeline".to_string()),
            source_report_path: report.output_path.clone(),
            evidence_path: row.evidence_path.clone(),
            stage_result_manifest_path: None,
            normalized_metric_count: row.metrics.len(),
            output_count: row.outputs.len() + usize::from(row.evidence_path.is_some()),
            log_count: 0,
        });
    }
    Ok(())
}

fn collect_core_germline_pipeline_rows(
    repo_root: &Path,
    report: &CoreGermlineMicroPipelineReport,
    result_rows: &mut Vec<MicroBenchmarkResultRow>,
    output_rows: &mut Vec<MicroBenchmarkOutputRow>,
) -> Result<()> {
    for row in &report.rows {
        output_rows.push(MicroBenchmarkOutputRow {
            execution_id: row.stage_id.clone(),
            component_id: "core_germline_micro_pipeline".to_string(),
            artifact_id: "evidence".to_string(),
            role: "pipeline_stage_evidence".to_string(),
            path: row.evidence_path.clone(),
            exists: repo_root.join(&row.evidence_path).is_file(),
            source: "core_germline_micro_pipeline".to_string(),
        });

        for (artifact_id, path) in &row.outputs {
            output_rows.push(MicroBenchmarkOutputRow {
                execution_id: row.stage_id.clone(),
                component_id: "core_germline_micro_pipeline".to_string(),
                artifact_id: artifact_id.clone(),
                role: "pipeline_stage_output".to_string(),
                path: path.clone(),
                exists: repo_root.join(path).exists(),
                source: "core_germline_micro_pipeline".to_string(),
            });
        }

        result_rows.push(MicroBenchmarkResultRow {
            execution_id: row.stage_id.clone(),
            component_id: "core_germline_micro_pipeline".to_string(),
            result_kind: if row.domain == "vcf" {
                MicroBenchmarkResultKind::PipelineBridge
            } else {
                MicroBenchmarkResultKind::Stage
            },
            domain: row.domain.clone(),
            bridge_source_domain: None,
            bridge_target_domain: None,
            stage_id: row.stage_id.clone(),
            tool_id: row.tool_id.clone(),
            status: MicroBenchmarkExecutionStatus::Succeeded,
            reason: "core_germline_pipeline_execution".to_string(),
            command: Some("bijux-dna bench local run-core-germline-micro-pipeline".to_string()),
            source_report_path: report.output_path.clone(),
            evidence_path: Some(row.evidence_path.clone()),
            stage_result_manifest_path: None,
            normalized_metric_count: row.metrics.len(),
            output_count: row.outputs.len() + 1,
            log_count: 0,
        });
    }
    Ok(())
}

fn append_manifest_outputs(
    repo_root: &Path,
    row: &super::local_real_smoke_core_subset::RealSmokeCoreSubsetRow,
    manifest: &BenchStageResultManifestV1,
    output_rows: &mut Vec<MicroBenchmarkOutputRow>,
    log_rows: &mut Vec<MicroBenchmarkLogRow>,
) -> (usize, usize) {
    let mut output_count = 0usize;
    let mut log_count = 0usize;
    for output in &manifest.outputs {
        let exists = repo_root.join(&output.realized_path).exists();
        output_rows.push(MicroBenchmarkOutputRow {
            execution_id: row.execution_id.clone(),
            component_id: "real_smoke_core_subset".to_string(),
            artifact_id: output.artifact_id.clone(),
            role: output.role.clone(),
            path: output.realized_path.clone(),
            exists,
            source: "stage_result_manifest".to_string(),
        });
        output_count += 1;
        if output.role.contains("log") {
            log_rows.push(MicroBenchmarkLogRow {
                execution_id: row.execution_id.clone(),
                component_id: "real_smoke_core_subset".to_string(),
                role: output.role.clone(),
                path: output.realized_path.clone(),
                exists,
                source: "stage_result_manifest".to_string(),
            });
            log_count += 1;
        }
    }
    (output_count, log_count)
}

fn ensure_micro_benchmark_run_contract(
    repo_root: &Path,
    manifest: &mut MicroBenchmarkRunManifest,
    result_rows: &[MicroBenchmarkResultRow],
    output_rows: &[MicroBenchmarkOutputRow],
    log_rows: &[MicroBenchmarkLogRow],
    normalized_metric_rows: &[MicroBenchmarkNormalizedMetricRow],
) -> Result<()> {
    if manifest.run_root != "runs/bench/micro" {
        bail!(
            "micro benchmark run root must stay `runs/bench/micro`, found `{}`",
            manifest.run_root
        );
    }
    if manifest.component_reports.len() < 6 {
        bail!(
            "micro benchmark run must keep at least six component reports, found {}",
            manifest.component_reports.len()
        );
    }
    if result_rows.is_empty() {
        bail!("micro benchmark run must write at least one result row");
    }
    if output_rows.is_empty() {
        bail!("micro benchmark run must write at least one output row");
    }
    if log_rows.is_empty() {
        bail!("micro benchmark run must write at least one log row");
    }
    if normalized_metric_rows.is_empty() {
        bail!("micro benchmark run must write at least one normalized metric row");
    }
    if manifest.result_row_count != result_rows.len()
        || manifest.output_row_count != output_rows.len()
        || manifest.log_row_count != log_rows.len()
        || manifest.normalized_metric_row_count != normalized_metric_rows.len()
    {
        bail!("micro benchmark manifest counts drifted from written row sets");
    }
    for component in &manifest.component_reports {
        let component_path = repo_root.join(&component.report_path);
        if !component_path.is_file() {
            bail!(
                "micro benchmark component report `{}` is missing at `{}`",
                component.component_id,
                component.report_path
            );
        }
        if !component.report_path.starts_with("runs/bench/micro/") {
            bail!(
                "micro benchmark component report `{}` must stay under `runs/bench/micro`, found `{}`",
                component.component_id,
                component.report_path
            );
        }
    }
    let component_ids = manifest
        .component_reports
        .iter()
        .map(|component| component.component_id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    let expected_component_ids = std::collections::BTreeSet::from([
        "adna_micro_pipeline",
        "bam_micro_smoke_subset",
        "core_germline_micro_pipeline",
        "fastq_micro_smoke_subset",
        "real_smoke_core_subset",
        "vcf_micro_smoke_subset",
    ]);
    if !expected_component_ids.is_subset(&component_ids) {
        bail!(
            "micro benchmark component reports drifted: observed={component_ids:?} expected_at_least={expected_component_ids:?}"
        );
    }
    for result_row in result_rows {
        match result_row.status {
            MicroBenchmarkExecutionStatus::Succeeded => {
                let evidence_path = result_row.evidence_path.as_ref().ok_or_else(|| {
                    anyhow!(
                        "successful micro benchmark row `{}` must keep an evidence path",
                        result_row.execution_id
                    )
                })?;
                if !repo_root.join(evidence_path).is_file() {
                    bail!(
                        "successful micro benchmark row `{}` is missing evidence `{}`",
                        result_row.execution_id,
                        evidence_path
                    );
                }
            }
            MicroBenchmarkExecutionStatus::ContainerNeeded
            | MicroBenchmarkExecutionStatus::Unavailable => {
                if result_row.normalized_metric_count != 0 {
                    bail!(
                        "non-succeeded micro benchmark row `{}` must not claim normalized metrics",
                        result_row.execution_id
                    );
                }
            }
        }
    }
    for output_row in output_rows {
        if !output_row.exists {
            bail!(
                "micro benchmark output row `{}` references missing path `{}`",
                output_row.execution_id,
                output_row.path
            );
        }
    }
    for log_row in log_rows {
        if !log_row.exists {
            bail!(
                "micro benchmark log row `{}` references missing path `{}`",
                log_row.execution_id,
                log_row.path
            );
        }
    }
    if !normalized_metric_rows
        .iter()
        .any(|row| row.execution_id == "vcf.call" || row.execution_id == "vcf.stats")
    {
        bail!("micro benchmark normalized metrics must include at least one VCF execution");
    }
    if !result_rows
        .iter()
        .any(|row| row.component_id == "fastq_micro_smoke_subset" && row.domain == "fastq")
    {
        bail!("micro benchmark results must include at least one FASTQ family representative");
    }
    if !result_rows
        .iter()
        .any(|row| row.component_id == "vcf_micro_smoke_subset" && row.domain == "vcf")
    {
        bail!("micro benchmark results must include at least one VCF family representative");
    }
    manifest.passes_behavior_test = true;
    Ok(())
}

fn real_result_kind(kind: RealSmokeCoreSubsetExecutionKind) -> MicroBenchmarkResultKind {
    match kind {
        RealSmokeCoreSubsetExecutionKind::Stage => MicroBenchmarkResultKind::Stage,
        RealSmokeCoreSubsetExecutionKind::PipelineBridge => {
            MicroBenchmarkResultKind::PipelineBridge
        }
    }
}

fn bam_result_status(status: BamMicroSmokeExecutionStatus) -> MicroBenchmarkExecutionStatus {
    match status {
        BamMicroSmokeExecutionStatus::LocalSmoke => MicroBenchmarkExecutionStatus::Succeeded,
        BamMicroSmokeExecutionStatus::ContainerNeeded => {
            MicroBenchmarkExecutionStatus::ContainerNeeded
        }
        BamMicroSmokeExecutionStatus::Unavailable => MicroBenchmarkExecutionStatus::Unavailable,
    }
}

fn fastq_result_status(status: FastqMicroSmokeExecutionStatus) -> MicroBenchmarkExecutionStatus {
    match status {
        FastqMicroSmokeExecutionStatus::LocalSmoke => MicroBenchmarkExecutionStatus::Succeeded,
        FastqMicroSmokeExecutionStatus::ContainerNeeded => {
            MicroBenchmarkExecutionStatus::ContainerNeeded
        }
        FastqMicroSmokeExecutionStatus::Unavailable => MicroBenchmarkExecutionStatus::Unavailable,
    }
}

fn vcf_result_status(status: VcfMicroSmokeExecutionStatus) -> MicroBenchmarkExecutionStatus {
    match status {
        VcfMicroSmokeExecutionStatus::LocalSmoke => MicroBenchmarkExecutionStatus::Succeeded,
        VcfMicroSmokeExecutionStatus::ContainerNeeded => {
            MicroBenchmarkExecutionStatus::ContainerNeeded
        }
        VcfMicroSmokeExecutionStatus::Unavailable => MicroBenchmarkExecutionStatus::Unavailable,
    }
}

fn build_run_id(
    repo_revision: &str,
    result_rows: &[MicroBenchmarkResultRow],
    output_rows: &[MicroBenchmarkOutputRow],
    log_rows: &[MicroBenchmarkLogRow],
    normalized_metric_rows: &[MicroBenchmarkNormalizedMetricRow],
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(MICRO_BENCHMARK_RUN_SCHEMA_VERSION.as_bytes());
    hasher.update(b"\n");
    hasher.update(repo_revision.as_bytes());
    hasher.update(b"\n");
    for row in result_rows {
        hasher.update(row.execution_id.as_bytes());
        hasher.update(b"\n");
        hasher.update(row.component_id.as_bytes());
        hasher.update(b"\n");
        hasher.update(row.stage_id.as_bytes());
        hasher.update(b"\n");
        hasher.update(row.tool_id.as_bytes());
        hasher.update(b"\n");
    }
    hasher.update(output_rows.len().to_string().as_bytes());
    hasher.update(log_rows.len().to_string().as_bytes());
    hasher.update(normalized_metric_rows.len().to_string().as_bytes());
    let digest = sha256_hex(&hasher.finalize());
    format!("micro-benchmark-{}", &digest[..12])
}

fn write_run_log(log_path: &Path, lines: &[String]) -> Result<()> {
    let mut file =
        fs::File::create(log_path).with_context(|| format!("create {}", log_path.display()))?;
    for line in lines {
        writeln!(file, "{line}").with_context(|| format!("write {}", log_path.display()))?;
    }
    Ok(())
}

fn git_stdout(repo_root: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .current_dir(repo_root)
        .args(args)
        .output()
        .with_context(|| format!("run git {}", args.join(" ")))?;
    if !output.status.success() {
        bail!("git {} failed: {}", args.join(" "), String::from_utf8_lossy(&output.stderr).trim());
    }
    Ok(String::from_utf8(output.stdout).context("decode git stdout")?.trim().to_string())
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(output, "{byte:02x}");
    }
    output
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn micro_benchmark_contract_accepts_governed_row_sets() {
        let temp = tempdir().expect("tempdir");
        let repo_root = temp.path();
        let component_core = repo_root.join("runs/bench/micro/core/REAL_SMOKE_CORE_SUMMARY.json");
        let component_fastq = repo_root.join("runs/bench/micro/fastq/MICRO_FASTQ_SUMMARY.json");
        let component_bam = repo_root.join("runs/bench/micro/bam/MICRO_BAM_SUMMARY.json");
        let component_vcf = repo_root.join("runs/bench/micro/vcf/MICRO_VCF_SUMMARY.json");
        let component_adna =
            repo_root.join("runs/bench/micro/pipelines/adna/MICRO_ADNA_SUMMARY.json");
        let component_pipeline =
            repo_root.join("runs/bench/micro/pipelines/core-germline/MICRO_PIPELINE_SUMMARY.json");
        let evidence_path =
            repo_root.join("runs/bench/local-smoke/vcf.stats/bcftools/metrics.json");
        let fastq_evidence_path =
            repo_root.join("runs/bench/local-smoke/fastq.validate_reads/report.json");
        let vcf_evidence_path =
            repo_root.join("runs/bench/local-smoke/vcf.call/bcftools/stage-result.json");
        let adna_evidence_path =
            repo_root.join("runs/bench/micro/pipelines/adna/artifacts/vcf.stats/report.json");
        let pipeline_evidence_path = repo_root
            .join("runs/bench/micro/pipelines/core-germline/artifacts/vcf.call/report.json");
        let output_path = repo_root.join("runs/bench/local-smoke/vcf.stats/bcftools/stats.txt");
        let log_path = repo_root.join("runs/bench/micro/logs/MICRO_RUN.log");
        let result_rows_path = repo_root.join("runs/bench/micro/results/MICRO_RESULT_ROWS.json");
        let output_rows_path = repo_root.join("runs/bench/micro/outputs/MICRO_OUTPUT_ROWS.json");
        let log_rows_path = repo_root.join("runs/bench/micro/logs/MICRO_LOG_ROWS.json");
        let normalized_metrics_path =
            repo_root.join("runs/bench/micro/normalized-metrics/MICRO_NORMALIZED_METRICS.json");

        for path in [
            &component_core,
            &component_fastq,
            &component_bam,
            &component_vcf,
            &component_adna,
            &component_pipeline,
            &evidence_path,
            &fastq_evidence_path,
            &vcf_evidence_path,
            &adna_evidence_path,
            &pipeline_evidence_path,
            &output_path,
            &log_path,
            &result_rows_path,
            &output_rows_path,
            &log_rows_path,
            &normalized_metrics_path,
        ] {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).expect("create parent");
            }
            std::fs::write(path, "{}").expect("write file");
        }

        let result_rows = vec![
            MicroBenchmarkResultRow {
                execution_id: "vcf.stats".to_string(),
                component_id: "real_smoke_core_subset".to_string(),
                result_kind: MicroBenchmarkResultKind::Stage,
                domain: "vcf".to_string(),
                bridge_source_domain: None,
                bridge_target_domain: None,
                stage_id: "vcf.stats".to_string(),
                tool_id: "bcftools".to_string(),
                status: MicroBenchmarkExecutionStatus::Succeeded,
                reason: "real_smoke_execution".to_string(),
                command: Some("bcftools stats".to_string()),
                source_report_path: "runs/bench/micro/core/REAL_SMOKE_CORE_SUMMARY.json"
                    .to_string(),
                evidence_path: Some(
                    "runs/bench/local-smoke/vcf.stats/bcftools/metrics.json".to_string(),
                ),
                stage_result_manifest_path: None,
                normalized_metric_count: 2,
                output_count: 1,
                log_count: 0,
            },
            MicroBenchmarkResultRow {
                execution_id: "fastq.validate_reads".to_string(),
                component_id: "fastq_micro_smoke_subset".to_string(),
                result_kind: MicroBenchmarkResultKind::FamilyRepresentative,
                domain: "fastq".to_string(),
                bridge_source_domain: None,
                bridge_target_domain: None,
                stage_id: "fastq.validate_reads".to_string(),
                tool_id: "fastq_scan".to_string(),
                status: MicroBenchmarkExecutionStatus::Succeeded,
                reason: "governed local smoke".to_string(),
                command: Some(
                    "bijux-dna bench local materialize-stage --stage-id fastq.validate_reads"
                        .to_string(),
                ),
                source_report_path: "runs/bench/micro/fastq/MICRO_FASTQ_SUMMARY.json".to_string(),
                evidence_path: Some(
                    "runs/bench/local-smoke/fastq.validate_reads/report.json".to_string(),
                ),
                stage_result_manifest_path: None,
                normalized_metric_count: 0,
                output_count: 1,
                log_count: 0,
            },
            MicroBenchmarkResultRow {
                execution_id: "bam.align".to_string(),
                component_id: "bam_micro_smoke_subset".to_string(),
                result_kind: MicroBenchmarkResultKind::FamilyRepresentative,
                domain: "bam".to_string(),
                bridge_source_domain: None,
                bridge_target_domain: None,
                stage_id: "bam.align".to_string(),
                tool_id: "bwa".to_string(),
                status: MicroBenchmarkExecutionStatus::ContainerNeeded,
                reason: "container image required".to_string(),
                command: Some(
                    "bijux-dna bench local run-bam-stage-smoke --stage-id bam.align".to_string(),
                ),
                source_report_path: "runs/bench/micro/bam/MICRO_BAM_SUMMARY.json".to_string(),
                evidence_path: None,
                stage_result_manifest_path: None,
                normalized_metric_count: 0,
                output_count: 0,
                log_count: 0,
            },
            MicroBenchmarkResultRow {
                execution_id: "vcf.calling".to_string(),
                component_id: "vcf_micro_smoke_subset".to_string(),
                result_kind: MicroBenchmarkResultKind::FamilyRepresentative,
                domain: "vcf".to_string(),
                bridge_source_domain: None,
                bridge_target_domain: None,
                stage_id: "vcf.call".to_string(),
                tool_id: "bcftools".to_string(),
                status: MicroBenchmarkExecutionStatus::Succeeded,
                reason: "governed local smoke".to_string(),
                command: Some(
                    "bijux-dna bench local run-vcf-call-smoke --tool-id bcftools".to_string(),
                ),
                source_report_path: "runs/bench/micro/vcf/MICRO_VCF_SUMMARY.json".to_string(),
                evidence_path: Some(
                    "runs/bench/local-smoke/vcf.call/bcftools/stage-result.json".to_string(),
                ),
                stage_result_manifest_path: None,
                normalized_metric_count: 0,
                output_count: 1,
                log_count: 0,
            },
            MicroBenchmarkResultRow {
                execution_id: "vcf.stats".to_string(),
                component_id: "adna_micro_pipeline".to_string(),
                result_kind: MicroBenchmarkResultKind::PipelineBridge,
                domain: "vcf".to_string(),
                bridge_source_domain: None,
                bridge_target_domain: None,
                stage_id: "vcf.stats".to_string(),
                tool_id: "bcftools".to_string(),
                status: MicroBenchmarkExecutionStatus::Succeeded,
                reason: "adna_micro_pipeline_execution".to_string(),
                command: Some("bijux-dna bench local run-adna-micro-pipeline".to_string()),
                source_report_path: "runs/bench/micro/pipelines/adna/MICRO_ADNA_SUMMARY.json"
                    .to_string(),
                evidence_path: Some(
                    "runs/bench/micro/pipelines/adna/artifacts/vcf.stats/report.json".to_string(),
                ),
                stage_result_manifest_path: None,
                normalized_metric_count: 3,
                output_count: 3,
                log_count: 0,
            },
            MicroBenchmarkResultRow {
                execution_id: "vcf.call".to_string(),
                component_id: "core_germline_micro_pipeline".to_string(),
                result_kind: MicroBenchmarkResultKind::PipelineBridge,
                domain: "vcf".to_string(),
                bridge_source_domain: None,
                bridge_target_domain: None,
                stage_id: "vcf.call".to_string(),
                tool_id: "bcftools".to_string(),
                status: MicroBenchmarkExecutionStatus::Succeeded,
                reason: "core_germline_pipeline_execution".to_string(),
                command: Some("bijux-dna bench local run-core-germline-micro-pipeline".to_string()),
                source_report_path:
                    "runs/bench/micro/pipelines/core-germline/MICRO_PIPELINE_SUMMARY.json"
                        .to_string(),
                evidence_path: Some(
                    "runs/bench/micro/pipelines/core-germline/artifacts/vcf.call/report.json"
                        .to_string(),
                ),
                stage_result_manifest_path: None,
                normalized_metric_count: 5,
                output_count: 2,
                log_count: 0,
            },
        ];
        let output_rows = vec![
            MicroBenchmarkOutputRow {
                execution_id: "vcf.stats".to_string(),
                component_id: "real_smoke_core_subset".to_string(),
                artifact_id: "evidence".to_string(),
                role: "evidence_report".to_string(),
                path: "runs/bench/local-smoke/vcf.stats/bcftools/stats.txt".to_string(),
                exists: true,
                source: "real_smoke_core_subset".to_string(),
            },
            MicroBenchmarkOutputRow {
                execution_id: "fastq.validate_reads".to_string(),
                component_id: "fastq_micro_smoke_subset".to_string(),
                artifact_id: "evidence".to_string(),
                role: "family_evidence".to_string(),
                path: "runs/bench/local-smoke/fastq.validate_reads/report.json".to_string(),
                exists: true,
                source: "fastq_micro_smoke_subset".to_string(),
            },
            MicroBenchmarkOutputRow {
                execution_id: "vcf.calling".to_string(),
                component_id: "vcf_micro_smoke_subset".to_string(),
                artifact_id: "evidence".to_string(),
                role: "family_evidence".to_string(),
                path: "runs/bench/local-smoke/vcf.call/bcftools/stage-result.json".to_string(),
                exists: true,
                source: "vcf_micro_smoke_subset".to_string(),
            },
            MicroBenchmarkOutputRow {
                execution_id: "vcf.stats".to_string(),
                component_id: "adna_micro_pipeline".to_string(),
                artifact_id: "evidence".to_string(),
                role: "pipeline_stage_evidence".to_string(),
                path: "runs/bench/micro/pipelines/adna/artifacts/vcf.stats/report.json".to_string(),
                exists: true,
                source: "adna_micro_pipeline".to_string(),
            },
            MicroBenchmarkOutputRow {
                execution_id: "vcf.call".to_string(),
                component_id: "core_germline_micro_pipeline".to_string(),
                artifact_id: "evidence".to_string(),
                role: "pipeline_stage_evidence".to_string(),
                path: "runs/bench/micro/pipelines/core-germline/artifacts/vcf.call/report.json"
                    .to_string(),
                exists: true,
                source: "core_germline_micro_pipeline".to_string(),
            },
        ];
        let log_rows = vec![MicroBenchmarkLogRow {
            execution_id: "micro.run".to_string(),
            component_id: "micro_benchmark_run".to_string(),
            role: "run_log".to_string(),
            path: "runs/bench/micro/logs/MICRO_RUN.log".to_string(),
            exists: true,
            source: "micro_benchmark_run".to_string(),
        }];
        let normalized_metric_rows = vec![
            MicroBenchmarkNormalizedMetricRow {
                execution_id: "vcf.stats".to_string(),
                component_id: "real_smoke_core_subset".to_string(),
                metric_id: "variant_count".to_string(),
                value: serde_json::json!(42),
                source_path: "runs/bench/local-smoke/vcf.stats/bcftools/metrics.json".to_string(),
            },
            MicroBenchmarkNormalizedMetricRow {
                execution_id: "vcf.stats".to_string(),
                component_id: "real_smoke_core_subset".to_string(),
                metric_id: "snp_count".to_string(),
                value: serde_json::json!(40),
                source_path: "runs/bench/local-smoke/vcf.stats/bcftools/metrics.json".to_string(),
            },
        ];
        let mut manifest = MicroBenchmarkRunManifest {
            schema_version: MICRO_BENCHMARK_RUN_SCHEMA_VERSION,
            manifest_path: DEFAULT_MICRO_BENCHMARK_RUN_MANIFEST_PATH.to_string(),
            run_root: "runs/bench/micro".to_string(),
            run_id: "micro-benchmark-123456789abc".to_string(),
            repo_revision: "0123456789abcdef0123456789abcdef01234567".to_string(),
            worktree_dirty: false,
            created_at_unix: 1,
            command: MICRO_BENCHMARK_RUN_COMMAND,
            component_reports: vec![
                MicroBenchmarkComponentReport {
                    component_id: "real_smoke_core_subset".to_string(),
                    report_path: "runs/bench/micro/core/REAL_SMOKE_CORE_SUMMARY.json".to_string(),
                    row_count: 1,
                },
                MicroBenchmarkComponentReport {
                    component_id: "fastq_micro_smoke_subset".to_string(),
                    report_path: "runs/bench/micro/fastq/MICRO_FASTQ_SUMMARY.json".to_string(),
                    row_count: 1,
                },
                MicroBenchmarkComponentReport {
                    component_id: "bam_micro_smoke_subset".to_string(),
                    report_path: "runs/bench/micro/bam/MICRO_BAM_SUMMARY.json".to_string(),
                    row_count: 1,
                },
                MicroBenchmarkComponentReport {
                    component_id: "vcf_micro_smoke_subset".to_string(),
                    report_path: "runs/bench/micro/vcf/MICRO_VCF_SUMMARY.json".to_string(),
                    row_count: 1,
                },
                MicroBenchmarkComponentReport {
                    component_id: "adna_micro_pipeline".to_string(),
                    report_path: "runs/bench/micro/pipelines/adna/MICRO_ADNA_SUMMARY.json"
                        .to_string(),
                    row_count: 15,
                },
                MicroBenchmarkComponentReport {
                    component_id: "core_germline_micro_pipeline".to_string(),
                    report_path:
                        "runs/bench/micro/pipelines/core-germline/MICRO_PIPELINE_SUMMARY.json"
                            .to_string(),
                    row_count: 12,
                },
            ],
            result_rows_path: "runs/bench/micro/results/MICRO_RESULT_ROWS.json".to_string(),
            output_rows_path: "runs/bench/micro/outputs/MICRO_OUTPUT_ROWS.json".to_string(),
            log_rows_path: "runs/bench/micro/logs/MICRO_LOG_ROWS.json".to_string(),
            normalized_metrics_path:
                "runs/bench/micro/normalized-metrics/MICRO_NORMALIZED_METRICS.json".to_string(),
            result_row_count: result_rows.len(),
            output_row_count: output_rows.len(),
            log_row_count: log_rows.len(),
            normalized_metric_row_count: normalized_metric_rows.len(),
            passes_behavior_test: false,
        };

        ensure_micro_benchmark_run_contract(
            repo_root,
            &mut manifest,
            &result_rows,
            &output_rows,
            &log_rows,
            &normalized_metric_rows,
        )
        .expect("micro contract");

        assert!(manifest.passes_behavior_test);
    }
}
