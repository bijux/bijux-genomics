use bijux_dna_api::v1::api::bench::{
    build_vcf_tool_score_rows, build_vcf_tool_scores_report as build_report,
    load_vcf_tool_score_evidence as load_vcf_evidence,
    VcfScoreExpectedRow as ExpectedBenchmarkReportRowView,
    VcfScoreMemoryRow as MicroBenchmarkMemoryRowView,
    VcfScoreRuntimeRow as MicroBenchmarkRuntimeRowView, VcfToolScoreRow, VcfToolScoreStatus,
    VcfToolScoresReport, VcfToolScoringInput,
};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde::Deserialize;

use super::all_domain_expected_benchmark_results::collect_all_domain_expected_benchmark_result_rows;
use super::catalog::load_external_artifact_binding_keys;
use super::stage_scoring::{StageScoringConfig, DEFAULT_STAGE_SCORING_PATH};
use crate::commands::benchmark::local_micro_benchmark_report::DEFAULT_MICRO_BENCHMARK_REPORT_JSON_PATH;
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_VCF_TOOL_SCORES_PATH: &str =
    "artifacts/benchmark-smoke/micro/vcf/VCF_TOOL_SCORES.tsv";

#[derive(Debug, Clone, Deserialize)]
struct ExpectedBenchmarkReportView {
    rows: Vec<ExpectedBenchmarkReportRowView>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct MicroBenchmarkReportView {
    runtime_rows: Vec<MicroBenchmarkRuntimeRowView>,
    memory_source_rows: Vec<MicroBenchmarkMemoryRowView>,
}

pub(crate) fn run_render_vcf_tool_scores(
    args: &parse::BenchReadinessRenderVcfToolScoresArgs,
) -> Result<()> {
    let repo_root = crate::commands::support::workspace_root::resolve_repo_root()?;
    let report = render_vcf_tool_scores(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_TOOL_SCORES_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn run_validate_vcf_tool_scores(
    args: &parse::BenchReadinessValidateVcfToolScoresArgs,
) -> Result<()> {
    let repo_root = crate::commands::support::workspace_root::resolve_repo_root()?;
    let report = validate_vcf_tool_scores(
        &repo_root,
        args.input.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_TOOL_SCORES_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_vcf_tool_scores(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<VcfToolScoresReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let (config_path, rows) = collect_vcf_tool_score_rows(repo_root)?;
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&output_path, render_vcf_tool_scores_tsv(&rows))
        .with_context(|| format!("write {}", output_path.display()))?;
    Ok(build_report(
        path_relative_to_repo(repo_root, &output_path),
        path_relative_to_repo(repo_root, &config_path),
        rows,
    ))
}

pub(crate) fn validate_vcf_tool_scores(
    repo_root: &Path,
    input_path: PathBuf,
) -> Result<VcfToolScoresReport> {
    let input_path = repo_relative_path(repo_root, &input_path);
    let actual = fs::read_to_string(&input_path)
        .with_context(|| format!("read {}", input_path.display()))?;
    let (config_path, rows) = collect_vcf_tool_score_rows(repo_root)?;
    let expected = render_vcf_tool_scores_tsv(&rows);
    if actual != expected {
        bail!(
            "VCF tool score TSV drifted from governed evidence contracts; rerun `bijux-dna bench readiness render-vcf-tool-scores`"
        );
    }
    Ok(build_report(
        path_relative_to_repo(repo_root, &input_path),
        path_relative_to_repo(repo_root, &config_path),
        rows,
    ))
}

fn collect_vcf_tool_score_rows(repo_root: &Path) -> Result<(PathBuf, Vec<VcfToolScoreRow>)> {
    let config_path = repo_root.join(DEFAULT_STAGE_SCORING_PATH);
    let config = load_stage_scoring_config(&config_path)?;
    let expected_report = collect_expected_benchmark_report(repo_root)?;
    let micro_report = load_micro_benchmark_report(repo_root)?;
    let evidence = load_vcf_evidence(repo_root)?;
    let external_artifact_bindings = load_external_artifact_binding_keys(repo_root)?;
    let rows = build_vcf_tool_score_rows(VcfToolScoringInput {
        config,
        expected_rows: expected_report.rows,
        runtime_rows: micro_report.runtime_rows,
        memory_rows: micro_report.memory_source_rows,
        evidence,
        external_artifact_bindings,
    })?;
    Ok((config_path, rows))
}

fn load_stage_scoring_config(path: &Path) -> Result<StageScoringConfig> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    toml::from_str(&raw).context("parse stage-scoring TOML")
}

fn collect_expected_benchmark_report(repo_root: &Path) -> Result<ExpectedBenchmarkReportView> {
    Ok(ExpectedBenchmarkReportView {
        rows: collect_all_domain_expected_benchmark_result_rows(repo_root)?
            .into_iter()
            .map(|row| ExpectedBenchmarkReportRowView {
                report_row_id: row.result_id.clone(),
                result_id: Some(row.result_id),
                domain: row.domain,
                stage_id: row.stage_id,
                tool_id: row.tool_id,
                corpus_id: row.corpus_id,
                report_section: row.report_section,
                row_status: "present".to_string(),
            })
            .collect(),
    })
}

fn load_micro_benchmark_report(repo_root: &Path) -> Result<MicroBenchmarkReportView> {
    let path = repo_root.join(DEFAULT_MICRO_BENCHMARK_REPORT_JSON_PATH);
    let raw = match fs::read_to_string(&path) {
        Ok(raw) => raw,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(MicroBenchmarkReportView::default());
        }
        Err(error) => return Err(error).with_context(|| format!("read {}", path.display())),
    };
    serde_json::from_str(&raw).context("parse micro benchmark report JSON")
}

fn render_vcf_tool_scores_tsv(rows: &[VcfToolScoreRow]) -> String {
    let mut lines = Vec::with_capacity(rows.len() + 1);
    lines.push(
        "stage_id\ttool_id\tdecision_mode\tcorrectness_signal\tresult_ids\treport_row_ids\tcorpus_ids\treport_sections\trow_statuses\tscore_status\ttruth_correctness_score\ttruth_correctness_basis\tcontract_correctness_score\tcontract_correctness_basis\tgenotype_truth_metric_value\tgenotype_truth_metric_basis\tmissingness_metric_value\tmissingness_metric_basis\tphasing_imputation_metric_value\tphasing_imputation_metric_basis\tpopulation_metric_value\tpopulation_metric_basis\tscientific_metric_ids\tscientific_metric_summary\truntime_seconds\truntime_source\tobserved_memory_mb\tdeclared_memory_mb\tmemory_source\tfailure_class\tmicro_execution_status\tscore_weight_coverage\tscore_total\tevidence_paths\treason".to_string(),
    );
    for row in rows {
        lines.push(format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            row.stage_id,
            row.tool_id,
            row.decision_mode,
            row.correctness_signal,
            join_csv(&row.result_ids),
            join_csv(&row.report_row_ids),
            join_csv(&row.corpus_ids),
            join_csv(&row.report_sections),
            join_csv(&row.row_statuses),
            score_status_label(row.score_status),
            format_optional_f64(row.truth_correctness_score),
            row.truth_correctness_basis.clone().unwrap_or_default(),
            format_optional_f64(row.contract_correctness_score),
            row.contract_correctness_basis.clone().unwrap_or_default(),
            format_optional_f64(row.genotype_truth_metric_value),
            row.genotype_truth_metric_basis.clone().unwrap_or_default(),
            format_optional_f64(row.missingness_metric_value),
            row.missingness_metric_basis.clone().unwrap_or_default(),
            format_optional_f64(row.phasing_imputation_metric_value),
            row.phasing_imputation_metric_basis.clone().unwrap_or_default(),
            format_optional_f64(row.population_metric_value),
            row.population_metric_basis.clone().unwrap_or_default(),
            join_csv(&row.scientific_metric_ids),
            sanitize_tsv_cell(row.scientific_metric_summary.as_deref().unwrap_or("")),
            format_optional_f64(row.runtime_seconds),
            row.runtime_source,
            format_optional_f64(row.observed_memory_mb),
            format_optional_f64(row.declared_memory_mb),
            row.memory_source,
            row.failure_class,
            row.micro_execution_status.clone().unwrap_or_default(),
            format_f64(row.score_weight_coverage),
            format_optional_f64(row.score_total),
            join_csv(&row.evidence_paths),
            sanitize_tsv_cell(&row.reason),
        ));
    }
    lines.join("\n") + "\n"
}

fn score_status_label(status: VcfToolScoreStatus) -> &'static str {
    match status {
        VcfToolScoreStatus::Scored => "scored",
        VcfToolScoreStatus::InsufficientEvidence => "insufficient_evidence",
        VcfToolScoreStatus::Blocked => "blocked",
    }
}

fn join_csv(values: &[String]) -> String {
    values.join(",")
}

fn format_optional_f64(value: Option<f64>) -> String {
    value.map(format_f64).unwrap_or_default()
}

fn format_f64(value: f64) -> String {
    format!("{value:.6}")
}

fn sanitize_tsv_cell(value: &str) -> String {
    value.replace(['\t', '\n'], " ")
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
