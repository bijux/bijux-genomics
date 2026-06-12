use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use super::corpus_asset_coverage_gate::{
    render_corpus_asset_coverage_gate, AssetAssignmentStatus, CorpusAssetCoverageGateReport,
};
use super::corpus_centric_report::DEFAULT_CORPUS_CENTRIC_REPORT_PATH;
use super::expected_benchmark_results::{
    collect_expected_benchmark_result_rows, DEFAULT_EXPECTED_BENCHMARK_RESULTS_PATH,
};
use super::fastq_corpus_assignment::{
    collect_fastq_corpus_assignment_rows, FastqCorpusAssignmentStatus,
};
use super::pair_readiness::{
    collect_pair_readiness_rows, PairAssetStatus, PairReadinessGap, PairReadinessRow,
    DEFAULT_PAIR_READINESS_PATH,
};
use super::parser_completeness_gate::{
    render_parser_completeness_gate, ParserCompletenessGateReport,
    DEFAULT_PARSER_COMPLETENESS_GATE_PATH,
};
use super::stage_centric_report::{
    collect_stage_centric_stage_reports, DEFAULT_STAGE_CENTRIC_REPORT_PATH,
};
use super::tool_centric_report::DEFAULT_TOOL_CENTRIC_REPORT_PATH;
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_BENCHMARK_READINESS_DASHBOARD_MARKDOWN_PATH: &str =
    "benchmarks/readiness/FASTQ_BAM_BENCHMARK_READINESS.md";
pub(crate) const DEFAULT_BENCHMARK_READINESS_DASHBOARD_JSON_PATH: &str =
    "benchmarks/readiness/FASTQ_BAM_BENCHMARK_READINESS.json";
const BENCHMARK_READINESS_DASHBOARD_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.benchmark_readiness_dashboard.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum DashboardSurfaceStatus {
    Complete,
    AttentionRequired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct BenchmarkReadinessMatrixSummary {
    pub(crate) surface_status: DashboardSurfaceStatus,
    pub(crate) scope: String,
    pub(crate) output_path: String,
    pub(crate) expected_pair_count: usize,
    pub(crate) ready_pair_count: usize,
    pub(crate) blocked_pair_count: usize,
    pub(crate) stage_count: usize,
    pub(crate) tool_count: usize,
    pub(crate) domain_counts: BTreeMap<String, usize>,
    pub(crate) readiness_gap_counts: BTreeMap<String, usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct BenchmarkReadinessAdapterSummary {
    pub(crate) surface_status: DashboardSurfaceStatus,
    pub(crate) scope: String,
    pub(crate) total_pair_count: usize,
    pub(crate) ready_pair_count: usize,
    pub(crate) attention_required_pair_count: usize,
    pub(crate) status_counts: BTreeMap<String, usize>,
    pub(crate) output_paths: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct BenchmarkReadinessParserSummary {
    pub(crate) surface_status: DashboardSurfaceStatus,
    pub(crate) scope: String,
    pub(crate) output_path: String,
    pub(crate) benchmark_reporting_pair_count: usize,
    pub(crate) ready_pair_count: usize,
    pub(crate) blocked_pair_count: usize,
    pub(crate) excluded_pair_count: usize,
    pub(crate) status_counts: BTreeMap<String, usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct BenchmarkReadinessCorpusSummary {
    pub(crate) surface_status: DashboardSurfaceStatus,
    pub(crate) scope: String,
    pub(crate) total_pair_count: usize,
    pub(crate) ready_pair_count: usize,
    pub(crate) blocked_pair_count: usize,
    pub(crate) corpus_family_count: usize,
    pub(crate) corpus_family_ids: Vec<String>,
    pub(crate) assigned_stage_count: usize,
    pub(crate) status_counts: BTreeMap<String, usize>,
    pub(crate) output_paths: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct BenchmarkReadinessAssetSummary {
    pub(crate) surface_status: DashboardSurfaceStatus,
    pub(crate) scope: String,
    pub(crate) output_path: String,
    pub(crate) asset_required_pair_count: usize,
    pub(crate) ready_pair_count: usize,
    pub(crate) blocked_pair_count: usize,
    pub(crate) overall_status_counts: BTreeMap<String, usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct BenchmarkReadinessReportSurfaceSummary {
    pub(crate) surface_status: DashboardSurfaceStatus,
    pub(crate) scope: String,
    pub(crate) output_count: usize,
    pub(crate) ready_output_count: usize,
    pub(crate) blocked_output_count: usize,
    pub(crate) expected_result_row_count: usize,
    pub(crate) stage_section_count: usize,
    pub(crate) tool_section_count: usize,
    pub(crate) corpus_section_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct BenchmarkReadinessReportOutput {
    pub(crate) report_id: String,
    pub(crate) output_path: String,
    pub(crate) item_kind: String,
    pub(crate) item_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct BenchmarkReadinessBlockedPair {
    pub(crate) row_id: String,
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) readiness_gap: String,
    pub(crate) support_status: String,
    pub(crate) adapter_status: String,
    pub(crate) parser_status: String,
    pub(crate) corpus_status: String,
    pub(crate) asset_status: String,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BenchmarkReadinessDashboardReport {
    pub(crate) schema_version: &'static str,
    pub(crate) markdown_output_path: String,
    pub(crate) json_output_path: String,
    pub(crate) expected_pair_count: usize,
    pub(crate) ready_pair_count: usize,
    pub(crate) blocked_pair_count: usize,
    pub(crate) blocker_counts: BTreeMap<String, usize>,
    pub(crate) matrix: BenchmarkReadinessMatrixSummary,
    pub(crate) adapters: BenchmarkReadinessAdapterSummary,
    pub(crate) parsers: BenchmarkReadinessParserSummary,
    pub(crate) corpora: BenchmarkReadinessCorpusSummary,
    pub(crate) assets: BenchmarkReadinessAssetSummary,
    pub(crate) reports: BenchmarkReadinessReportSurfaceSummary,
    pub(crate) report_outputs: Vec<BenchmarkReadinessReportOutput>,
    pub(crate) blocked_pairs: Vec<BenchmarkReadinessBlockedPair>,
}

pub(crate) fn run_render_benchmark_readiness_dashboard(
    args: &parse::BenchReadinessRenderBenchmarkReadinessDashboardArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_benchmark_readiness_dashboard(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_BENCHMARK_READINESS_DASHBOARD_MARKDOWN_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.markdown_output_path);
    }
    Ok(())
}

pub(crate) fn render_benchmark_readiness_dashboard(
    repo_root: &Path,
    markdown_output_path: PathBuf,
) -> Result<BenchmarkReadinessDashboardReport> {
    let markdown_output_path = repo_relative_path(repo_root, &markdown_output_path);
    let json_output_path = derive_json_output_path(&markdown_output_path);

    let pair_rows = collect_pair_readiness_rows(repo_root)?;
    let stage_reports = collect_stage_centric_stage_reports(repo_root)?;
    let expected_result_rows = collect_expected_benchmark_result_rows(repo_root)?;
    let parser_gate = render_parser_completeness_gate(
        repo_root,
        repo_root.join(DEFAULT_PARSER_COMPLETENESS_GATE_PATH),
    )?;
    let corpus_asset_gate = render_corpus_asset_coverage_gate(
        repo_root,
        repo_root.join(super::corpus_asset_coverage_gate::DEFAULT_CORPUS_ASSET_COVERAGE_GATE_PATH),
    )?;
    let matrix = summarize_matrix(&pair_rows, &stage_reports);
    let adapters = summarize_adapters(&pair_rows);
    let parsers = summarize_parsers(&parser_gate);
    let corpora = summarize_corpora(repo_root, &pair_rows)?;
    let assets = summarize_assets(&corpus_asset_gate);
    let reports = summarize_reports(&pair_rows, &stage_reports, &expected_result_rows, &corpora);
    let report_outputs = build_report_outputs(
        matrix.expected_pair_count,
        reports.expected_result_row_count,
        reports.stage_section_count,
        reports.tool_section_count,
        reports.corpus_section_count,
    );
    let blocked_pairs = pair_rows
        .iter()
        .filter(|row| row.benchmark_status != "benchmark_ready")
        .map(render_blocked_pair)
        .collect::<Vec<_>>();
    let mut blocker_counts = BTreeMap::<String, usize>::new();
    for row in &blocked_pairs {
        *blocker_counts.entry(row.readiness_gap.clone()).or_default() += 1;
    }

    let report = BenchmarkReadinessDashboardReport {
        schema_version: BENCHMARK_READINESS_DASHBOARD_SCHEMA_VERSION,
        markdown_output_path: path_relative_to_repo(repo_root, &markdown_output_path),
        json_output_path: path_relative_to_repo(repo_root, &json_output_path),
        expected_pair_count: matrix.expected_pair_count,
        ready_pair_count: matrix.ready_pair_count,
        blocked_pair_count: matrix.blocked_pair_count,
        blocker_counts,
        matrix,
        adapters,
        parsers,
        corpora,
        assets,
        reports,
        report_outputs,
        blocked_pairs,
    };
    ensure_benchmark_readiness_dashboard_contract(&report)?;

    if let Some(parent) = markdown_output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&markdown_output_path, render_benchmark_readiness_dashboard_markdown(&report))
        .with_context(|| format!("write {}", markdown_output_path.display()))?;
    let payload = serde_json::to_string_pretty(&report)
        .context("render benchmark readiness dashboard to JSON")?;
    bijux_dna_infra::atomic_write_bytes(&json_output_path, payload.as_bytes())?;
    Ok(report)
}

fn summarize_matrix(
    pair_rows: &[PairReadinessRow],
    stage_reports: &[super::stage_centric_report::StageCentricStageReport],
) -> BenchmarkReadinessMatrixSummary {
    let expected_pair_count = pair_rows.len();
    let ready_pair_count =
        pair_rows.iter().filter(|row| row.benchmark_status == "benchmark_ready").count();
    let blocked_pair_count = expected_pair_count.saturating_sub(ready_pair_count);
    let tool_count = pair_rows.iter().map(|row| row.tool_id.clone()).collect::<BTreeSet<_>>().len();
    let mut domain_counts = BTreeMap::<String, usize>::new();
    let mut readiness_gap_counts = BTreeMap::<String, usize>::new();
    for row in pair_rows {
        *domain_counts.entry(row.domain.clone()).or_default() += 1;
        *readiness_gap_counts
            .entry(pair_readiness_gap_label(row.readiness_gap).to_string())
            .or_default() += 1;
    }

    BenchmarkReadinessMatrixSummary {
        surface_status: if blocked_pair_count == 0 {
            DashboardSurfaceStatus::Complete
        } else {
            DashboardSurfaceStatus::AttentionRequired
        },
        scope: "all governed fastq and bam stage-tool pairs".to_string(),
        output_path: DEFAULT_PAIR_READINESS_PATH.to_string(),
        expected_pair_count,
        ready_pair_count,
        blocked_pair_count,
        stage_count: stage_reports.len(),
        tool_count,
        domain_counts,
        readiness_gap_counts,
    }
}

fn summarize_adapters(pair_rows: &[PairReadinessRow]) -> BenchmarkReadinessAdapterSummary {
    let mut status_counts = BTreeMap::<String, usize>::new();
    for row in pair_rows {
        *status_counts.entry(row.adapter_status.clone()).or_default() += 1;
    }
    let ready_pair_count = pair_rows
        .iter()
        .filter(|row| row.adapter_status == "runnable" || row.adapter_status == "plannable")
        .count();
    let attention_required_pair_count = pair_rows.len().saturating_sub(ready_pair_count);

    BenchmarkReadinessAdapterSummary {
        surface_status: if attention_required_pair_count == 0 {
            DashboardSurfaceStatus::Complete
        } else {
            DashboardSurfaceStatus::AttentionRequired
        },
        scope: "all governed fastq and bam stage-tool pairs".to_string(),
        total_pair_count: pair_rows.len(),
        ready_pair_count,
        attention_required_pair_count,
        status_counts,
        output_paths: vec![
            "benchmarks/readiness/fastq-command-adapter-coverage.tsv".to_string(),
            "benchmarks/readiness/bam-command-adapter-coverage.tsv".to_string(),
        ],
    }
}

fn summarize_parsers(report: &ParserCompletenessGateReport) -> BenchmarkReadinessParserSummary {
    let mut status_counts = BTreeMap::<String, usize>::new();
    for row in &report.rows {
        *status_counts.entry(row.parser_status.clone()).or_default() += 1;
    }

    BenchmarkReadinessParserSummary {
        surface_status: if report.gate_failed_row_count == 0 {
            DashboardSurfaceStatus::Complete
        } else {
            DashboardSurfaceStatus::AttentionRequired
        },
        scope: "benchmark-reporting pairs only".to_string(),
        output_path: DEFAULT_PARSER_COMPLETENESS_GATE_PATH.to_string(),
        benchmark_reporting_pair_count: report.gate_row_count,
        ready_pair_count: report.gate_passed_row_count,
        blocked_pair_count: report.gate_failed_row_count,
        excluded_pair_count: report.excluded_row_count,
        status_counts,
    }
}

fn summarize_corpora(
    repo_root: &Path,
    pair_rows: &[PairReadinessRow],
) -> Result<BenchmarkReadinessCorpusSummary> {
    let (_, _, fastq_rows) = collect_fastq_corpus_assignment_rows(repo_root)?;
    let (_, _, bam_rows) =
        super::bam_corpus_assignment::collect_bam_corpus_assignment_rows(repo_root)?;
    let mut corpus_family_ids = fastq_rows
        .into_iter()
        .filter(|row| row.assignment_status == FastqCorpusAssignmentStatus::Assigned)
        .filter_map(|row| row.corpus_family_id)
        .chain(bam_rows.into_iter().map(|row| row.corpus_family_id))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    corpus_family_ids.sort();
    let assigned_stage_count = pair_rows
        .iter()
        .filter(|row| row.corpus_status.starts_with("fixture:"))
        .map(|row| (row.domain.clone(), row.stage_id.clone()))
        .collect::<BTreeSet<_>>()
        .len();
    let mut status_counts = BTreeMap::<String, usize>::new();
    for row in pair_rows {
        *status_counts.entry(row.corpus_status.clone()).or_default() += 1;
    }
    let ready_pair_count =
        pair_rows.iter().filter(|row| row.corpus_status.starts_with("fixture:")).count();
    let blocked_pair_count = pair_rows.len().saturating_sub(ready_pair_count);

    Ok(BenchmarkReadinessCorpusSummary {
        surface_status: if blocked_pair_count == 0 {
            DashboardSurfaceStatus::Complete
        } else {
            DashboardSurfaceStatus::AttentionRequired
        },
        scope: "all governed fastq and bam stage-tool pairs".to_string(),
        total_pair_count: pair_rows.len(),
        ready_pair_count,
        blocked_pair_count,
        corpus_family_count: corpus_family_ids.len(),
        corpus_family_ids,
        assigned_stage_count,
        status_counts,
        output_paths: vec![
            "benchmarks/readiness/fastq-corpus-assignment.tsv".to_string(),
            "benchmarks/readiness/bam-corpus-assignment.tsv".to_string(),
            DEFAULT_CORPUS_CENTRIC_REPORT_PATH.to_string(),
        ],
    })
}

fn summarize_assets(report: &CorpusAssetCoverageGateReport) -> BenchmarkReadinessAssetSummary {
    let mut overall_status_counts = BTreeMap::<String, usize>::new();
    for row in &report.rows {
        *overall_status_counts
            .entry(asset_assignment_status_label(row.asset_assignment_status).to_string())
            .or_default() += 1;
    }

    BenchmarkReadinessAssetSummary {
        surface_status: if report.benchmark_ready_asset_missing_row_count == 0 {
            DashboardSurfaceStatus::Complete
        } else {
            DashboardSurfaceStatus::AttentionRequired
        },
        scope: "asset-required benchmark-submission pairs".to_string(),
        output_path: super::corpus_asset_coverage_gate::DEFAULT_CORPUS_ASSET_COVERAGE_GATE_PATH
            .to_string(),
        asset_required_pair_count: report.benchmark_ready_asset_required_row_count,
        ready_pair_count: report.benchmark_ready_asset_assigned_row_count,
        blocked_pair_count: report.benchmark_ready_asset_missing_row_count,
        overall_status_counts,
    }
}

fn summarize_reports(
    pair_rows: &[PairReadinessRow],
    stage_reports: &[super::stage_centric_report::StageCentricStageReport],
    expected_result_rows: &[super::expected_benchmark_results::ExpectedBenchmarkResultRow],
    corpora: &BenchmarkReadinessCorpusSummary,
) -> BenchmarkReadinessReportSurfaceSummary {
    let output_count = 5_usize;
    let ready_output_count = if !expected_result_rows.is_empty()
        && !stage_reports.is_empty()
        && !pair_rows.is_empty()
        && corpora.corpus_family_count > 0
    {
        output_count
    } else {
        0
    };
    let blocked_output_count = output_count.saturating_sub(ready_output_count);

    BenchmarkReadinessReportSurfaceSummary {
        surface_status: if blocked_output_count == 0 {
            DashboardSurfaceStatus::Complete
        } else {
            DashboardSurfaceStatus::AttentionRequired
        },
        scope: "governed local report surfaces".to_string(),
        output_count,
        ready_output_count,
        blocked_output_count,
        expected_result_row_count: expected_result_rows.len(),
        stage_section_count: stage_reports.len(),
        tool_section_count: pair_rows
            .iter()
            .map(|row| row.tool_id.clone())
            .collect::<BTreeSet<_>>()
            .len(),
        corpus_section_count: corpora.corpus_family_count,
    }
}

fn build_report_outputs(
    pair_count: usize,
    expected_result_row_count: usize,
    stage_section_count: usize,
    tool_section_count: usize,
    corpus_section_count: usize,
) -> Vec<BenchmarkReadinessReportOutput> {
    vec![
        BenchmarkReadinessReportOutput {
            report_id: "pair_readiness".to_string(),
            output_path: DEFAULT_PAIR_READINESS_PATH.to_string(),
            item_kind: "stage_tool_pairs".to_string(),
            item_count: pair_count,
        },
        BenchmarkReadinessReportOutput {
            report_id: "expected_benchmark_results".to_string(),
            output_path: DEFAULT_EXPECTED_BENCHMARK_RESULTS_PATH.to_string(),
            item_kind: "expected_results".to_string(),
            item_count: expected_result_row_count,
        },
        BenchmarkReadinessReportOutput {
            report_id: "stage_centric_report".to_string(),
            output_path: DEFAULT_STAGE_CENTRIC_REPORT_PATH.to_string(),
            item_kind: "stage_sections".to_string(),
            item_count: stage_section_count,
        },
        BenchmarkReadinessReportOutput {
            report_id: "tool_centric_report".to_string(),
            output_path: DEFAULT_TOOL_CENTRIC_REPORT_PATH.to_string(),
            item_kind: "tool_sections".to_string(),
            item_count: tool_section_count,
        },
        BenchmarkReadinessReportOutput {
            report_id: "corpus_centric_report".to_string(),
            output_path: DEFAULT_CORPUS_CENTRIC_REPORT_PATH.to_string(),
            item_kind: "corpus_sections".to_string(),
            item_count: corpus_section_count,
        },
    ]
}

fn render_blocked_pair(row: &PairReadinessRow) -> BenchmarkReadinessBlockedPair {
    BenchmarkReadinessBlockedPair {
        row_id: format!("{}:{}:{}", row.domain, row.stage_id, row.tool_id),
        domain: row.domain.clone(),
        stage_id: row.stage_id.clone(),
        tool_id: row.tool_id.clone(),
        readiness_gap: pair_readiness_gap_label(row.readiness_gap).to_string(),
        support_status: row.support_status.clone(),
        adapter_status: row.adapter_status.clone(),
        parser_status: row.parser_status.clone(),
        corpus_status: row.corpus_status.clone(),
        asset_status: pair_asset_status_label(row.asset_status).to_string(),
        reason: row.reason.clone(),
    }
}

fn ensure_benchmark_readiness_dashboard_contract(
    report: &BenchmarkReadinessDashboardReport,
) -> Result<()> {
    if report.expected_pair_count != 123
        || report.ready_pair_count != 116
        || report.blocked_pair_count != 7
    {
        return Err(anyhow!(
            "benchmark readiness dashboard pair summary drifted from the governed contract"
        ));
    }
    if report.matrix.stage_count != 51 || report.matrix.tool_count != 67 {
        return Err(anyhow!(
            "benchmark readiness dashboard matrix inventory drifted from the governed contract"
        ));
    }
    if report.blocker_counts.get("corpus").copied().unwrap_or_default() != 3
        || report.blocker_counts.get("support").copied().unwrap_or_default() != 4
    {
        return Err(anyhow!(
            "benchmark readiness dashboard blocker counts drifted from the governed contract"
        ));
    }
    if report.adapters.attention_required_pair_count != 4
        || report.parsers.benchmark_reporting_pair_count != 116
        || report.parsers.blocked_pair_count != 0
        || report.corpora.corpus_family_count != 7
        || report.corpora.assigned_stage_count != 49
        || report.corpora.blocked_pair_count != 3
        || report.assets.asset_required_pair_count != 18
        || report.assets.blocked_pair_count != 0
        || report.reports.expected_result_row_count != 116
        || report.reports.stage_section_count != 51
        || report.reports.tool_section_count != 67
        || report.reports.corpus_section_count != 7
    {
        return Err(anyhow!(
            "benchmark readiness dashboard surface summary drifted from the governed contract"
        ));
    }
    ensure_blocked_pair(report, "fastq:fastq.index_reference:bowtie2_build", "corpus")?;
    ensure_blocked_pair(report, "fastq:fastq.trim_reads:seqpurge", "support")?;
    ensure_blocked_pair(report, "fastq:fastq.report_qc:multiqc", "corpus")?;
    Ok(())
}

fn ensure_blocked_pair(
    report: &BenchmarkReadinessDashboardReport,
    row_id: &str,
    expected_gap: &str,
) -> Result<()> {
    let row =
        report.blocked_pairs.iter().find(|row| row.row_id == row_id).ok_or_else(|| {
            anyhow!("benchmark readiness dashboard is missing blocker `{row_id}`")
        })?;
    if row.readiness_gap != expected_gap {
        return Err(anyhow!(
            "benchmark readiness dashboard blocker `{row_id}` drifted from its governed gap"
        ));
    }
    Ok(())
}

fn render_benchmark_readiness_dashboard_markdown(
    report: &BenchmarkReadinessDashboardReport,
) -> String {
    let mut rendered = String::from("# FASTQ + BAM Benchmark Readiness Dashboard\n\n");
    rendered.push_str("## Summary\n\n");
    rendered.push_str(&format!(
        "- Expected pairs: {}\n- Ready pairs: {}\n- Blocked pairs: {}\n- Exact blocker counts: {}\n\n",
        report.expected_pair_count,
        report.ready_pair_count,
        report.blocked_pair_count,
        render_count_map(&report.blocker_counts),
    ));

    rendered.push_str("## Surface Summary\n\n");
    rendered.push_str("| Surface | Status | Scope | Total | Ready | Blocked | Evidence |\n");
    rendered.push_str("| --- | --- | --- | ---: | ---: | ---: | --- |\n");
    rendered.push_str(&format!(
        "| Matrix | {} | {} | {} | {} | {} | {} |\n",
        dashboard_surface_status_label(report.matrix.surface_status),
        sanitize_markdown_cell(&report.matrix.scope),
        report.matrix.expected_pair_count,
        report.matrix.ready_pair_count,
        report.matrix.blocked_pair_count,
        sanitize_markdown_cell(&format!(
            "stages={}, tools={}, gaps={}",
            report.matrix.stage_count,
            report.matrix.tool_count,
            render_count_map(&report.matrix.readiness_gap_counts)
        )),
    ));
    rendered.push_str(&format!(
        "| Adapters | {} | {} | {} | {} | {} | {} |\n",
        dashboard_surface_status_label(report.adapters.surface_status),
        sanitize_markdown_cell(&report.adapters.scope),
        report.adapters.total_pair_count,
        report.adapters.ready_pair_count,
        report.adapters.attention_required_pair_count,
        sanitize_markdown_cell(&render_count_map(&report.adapters.status_counts)),
    ));
    rendered.push_str(&format!(
        "| Parsers | {} | {} | {} | {} | {} | {} |\n",
        dashboard_surface_status_label(report.parsers.surface_status),
        sanitize_markdown_cell(&report.parsers.scope),
        report.parsers.benchmark_reporting_pair_count,
        report.parsers.ready_pair_count,
        report.parsers.blocked_pair_count,
        sanitize_markdown_cell(&format!(
            "excluded={}, statuses={}",
            report.parsers.excluded_pair_count,
            render_count_map(&report.parsers.status_counts)
        )),
    ));
    rendered.push_str(&format!(
        "| Corpora | {} | {} | {} | {} | {} | {} |\n",
        dashboard_surface_status_label(report.corpora.surface_status),
        sanitize_markdown_cell(&report.corpora.scope),
        report.corpora.total_pair_count,
        report.corpora.ready_pair_count,
        report.corpora.blocked_pair_count,
        sanitize_markdown_cell(&format!(
            "corpora={}, assigned stages={}, statuses={}",
            report.corpora.corpus_family_count,
            report.corpora.assigned_stage_count,
            render_count_map(&report.corpora.status_counts)
        )),
    ));
    rendered.push_str(&format!(
        "| Assets | {} | {} | {} | {} | {} | {} |\n",
        dashboard_surface_status_label(report.assets.surface_status),
        sanitize_markdown_cell(&report.assets.scope),
        report.assets.asset_required_pair_count,
        report.assets.ready_pair_count,
        report.assets.blocked_pair_count,
        sanitize_markdown_cell(&render_count_map(&report.assets.overall_status_counts)),
    ));
    rendered.push_str(&format!(
        "| Reports | {} | {} | {} | {} | {} | {} |\n",
        dashboard_surface_status_label(report.reports.surface_status),
        sanitize_markdown_cell(&report.reports.scope),
        report.reports.output_count,
        report.reports.ready_output_count,
        report.reports.blocked_output_count,
        sanitize_markdown_cell(&format!(
            "expected_results={}, stage_sections={}, tool_sections={}, corpus_sections={}",
            report.reports.expected_result_row_count,
            report.reports.stage_section_count,
            report.reports.tool_section_count,
            report.reports.corpus_section_count
        )),
    ));

    rendered.push_str("\n## Report Outputs\n\n");
    rendered.push_str("| Report | Output | Governed items |\n");
    rendered.push_str("| --- | --- | --- |\n");
    for output in &report.report_outputs {
        rendered.push_str(&format!(
            "| {} | {} | {} {} |\n",
            sanitize_markdown_cell(&output.report_id),
            sanitize_markdown_cell(&output.output_path),
            output.item_count,
            sanitize_markdown_cell(&output.item_kind),
        ));
    }

    rendered.push_str("\n## Exact Blockers\n\n");
    rendered.push_str(
        "| Domain | Stage | Tool | Gap | Support | Adapter | Parser | Corpus | Asset |\n",
    );
    rendered.push_str("| --- | --- | --- | --- | --- | --- | --- | --- | --- |\n");
    for row in &report.blocked_pairs {
        rendered.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
            sanitize_markdown_cell(&row.domain),
            sanitize_markdown_cell(&row.stage_id),
            sanitize_markdown_cell(&row.tool_id),
            sanitize_markdown_cell(&row.readiness_gap),
            sanitize_markdown_cell(&row.support_status),
            sanitize_markdown_cell(&row.adapter_status),
            sanitize_markdown_cell(&row.parser_status),
            sanitize_markdown_cell(&row.corpus_status),
            sanitize_markdown_cell(&row.asset_status),
        ));
    }

    rendered
}

fn render_count_map(counts: &BTreeMap<String, usize>) -> String {
    counts.iter().map(|(key, value)| format!("{key}={value}")).collect::<Vec<_>>().join(", ")
}

fn dashboard_surface_status_label(status: DashboardSurfaceStatus) -> &'static str {
    match status {
        DashboardSurfaceStatus::Complete => "complete",
        DashboardSurfaceStatus::AttentionRequired => "attention_required",
    }
}

fn pair_readiness_gap_label(gap: PairReadinessGap) -> &'static str {
    match gap {
        PairReadinessGap::None => "none",
        PairReadinessGap::Asset => "asset",
        PairReadinessGap::Corpus => "corpus",
        PairReadinessGap::Parser => "parser",
        PairReadinessGap::Adapter => "adapter",
        PairReadinessGap::Support => "support",
    }
}

fn pair_asset_status_label(status: PairAssetStatus) -> &'static str {
    match status {
        PairAssetStatus::Assigned => "assigned",
        PairAssetStatus::Missing => "missing",
        PairAssetStatus::NotRequired => "not_required",
    }
}

fn asset_assignment_status_label(status: AssetAssignmentStatus) -> &'static str {
    match status {
        AssetAssignmentStatus::Assigned => "assigned",
        AssetAssignmentStatus::Missing => "missing",
        AssetAssignmentStatus::NotRequired => "not_required",
    }
}

fn derive_json_output_path(markdown_output_path: &Path) -> PathBuf {
    match markdown_output_path.extension().and_then(|value| value.to_str()) {
        Some("md") => markdown_output_path.with_extension("json"),
        _ => markdown_output_path.with_extension("json"),
    }
}

fn repo_relative_path(repo_root: &Path, output_path: &Path) -> PathBuf {
    if output_path.is_absolute() {
        output_path.to_path_buf()
    } else {
        repo_root.join(output_path)
    }
}

fn path_relative_to_repo(repo_root: &Path, output_path: &Path) -> String {
    output_path.strip_prefix(repo_root).unwrap_or(output_path).to_string_lossy().replace('\\', "/")
}

fn sanitize_markdown_cell(value: &str) -> String {
    value.replace('|', "\\|").replace('\n', " ")
}

#[cfg(test)]
mod tests {
    use super::{
        render_benchmark_readiness_dashboard, DEFAULT_BENCHMARK_READINESS_DASHBOARD_MARKDOWN_PATH,
    };
    use std::path::PathBuf;

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("workspace crates directory")
            .parent()
            .expect("repository root")
            .to_path_buf()
    }

    #[test]
    fn benchmark_readiness_dashboard_tracks_governed_surface_counts() {
        let repo_root = repo_root();
        let tempdir = tempfile::tempdir().expect("tempdir");
        let report = render_benchmark_readiness_dashboard(
            &repo_root,
            tempdir.path().join(DEFAULT_BENCHMARK_READINESS_DASHBOARD_MARKDOWN_PATH),
        )
        .expect("render benchmark readiness dashboard");

        assert_eq!(report.expected_pair_count, 123);
        assert_eq!(report.ready_pair_count, 116);
        assert_eq!(report.blocked_pair_count, 7);
        assert_eq!(report.matrix.stage_count, 51);
        assert_eq!(report.matrix.tool_count, 67);
        assert_eq!(report.adapters.attention_required_pair_count, 4);
        assert_eq!(report.parsers.benchmark_reporting_pair_count, 116);
        assert_eq!(report.corpora.corpus_family_count, 7);
        assert_eq!(report.corpora.assigned_stage_count, 49);
        assert_eq!(report.assets.asset_required_pair_count, 18);
        assert_eq!(report.reports.expected_result_row_count, 116);
        assert_eq!(report.reports.stage_section_count, 51);
        assert_eq!(report.reports.tool_section_count, 67);
        assert_eq!(report.reports.corpus_section_count, 7);
        assert_eq!(report.blocked_pairs.len(), 7);
    }

    #[test]
    fn benchmark_readiness_dashboard_writes_governed_blocker_table() {
        let repo_root = repo_root();
        let tempdir = tempfile::tempdir().expect("tempdir");
        let output_path = tempdir.path().join(DEFAULT_BENCHMARK_READINESS_DASHBOARD_MARKDOWN_PATH);
        let report = render_benchmark_readiness_dashboard(&repo_root, output_path.clone())
            .expect("render benchmark readiness dashboard");

        assert!(report
            .markdown_output_path
            .ends_with(DEFAULT_BENCHMARK_READINESS_DASHBOARD_MARKDOWN_PATH));
        let markdown = std::fs::read_to_string(output_path).expect("read markdown");
        assert!(markdown.contains("# FASTQ + BAM Benchmark Readiness Dashboard"));
        assert!(markdown.contains("- Expected pairs: 123"));
        assert!(markdown.contains("- Ready pairs: 116"));
        assert!(markdown.contains("- Blocked pairs: 7"));
        assert!(markdown.contains("| Matrix | attention_required | all governed fastq and bam stage-tool pairs | 123 | 116 | 7 | stages=51, tools=67, gaps=corpus=3, none=116, support=4 |"));
        assert!(markdown.contains("| Corpora | attention_required | all governed fastq and bam stage-tool pairs | 123 | 120 | 3 | corpora=7, assigned stages=49, statuses=fixture:corpus-01-adna-bam-mini=7, fixture:corpus-01-adna-damage-mini=9, fixture:corpus-01-bam-mini=28, fixture:corpus-01-genotyping-mini=1, fixture:corpus-01-kinship-mini=2, fixture:corpus-01-mini=63, fixture:corpus-02-edna-mini=4, fixture:corpus-03-amplicon-mini=6, planner_only=3 |"));
        assert!(markdown.contains(
            "| pair_readiness | benchmarks/readiness/pair-readiness.tsv | 123 stage_tool_pairs |"
        ));
        assert!(markdown.contains("| corpus_centric_report | benchmarks/readiness/corpus-centric-report.md | 7 corpus_sections |"));
        assert!(markdown.contains("| fastq | fastq.index_reference | bowtie2_build | corpus | observer_specialized_benchmark | runnable | comparable | planner_only | assigned |"));
        assert!(markdown.contains("| fastq | fastq.trim_reads | seqpurge | support | planned_contract | declared_only | not_normalized | fixture:corpus-01-mini | not_required |"));
        assert!(markdown.contains("| fastq | fastq.report_qc | multiqc | corpus | observer_specialized_benchmark | runnable | comparable | planner_only | not_required |"));
    }
}
