use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use serde::Serialize;
use serde_json::Value;

use super::all_domain_expected_benchmark_results::{
    render_all_domain_expected_benchmark_results, AllDomainExpectedBenchmarkResultsReport,
    DEFAULT_ALL_DOMAIN_EXPECTED_BENCHMARK_RESULTS_PATH,
};
use super::all_domain_failure_classification::{
    render_all_domain_failure_classification, AllDomainFailureClassificationReport,
    DEFAULT_ALL_DOMAIN_FAILURE_CLASSIFICATION_PATH,
};
use super::all_domain_missing_result_test::{
    render_all_domain_missing_result_test, AllDomainMissingResultStatus,
    AllDomainMissingResultTestReport, DEFAULT_ALL_DOMAIN_MISSING_RESULT_TEST_PATH,
};
use crate::commands::benchmark::local_all_domain_fake_failures::{
    fake_run_all_domain_failures, AllDomainFakeFailuresManifest,
    DEFAULT_ALL_DOMAIN_FAKE_FAILURE_ROOT,
};
use crate::commands::benchmark::local_all_domain_fake_runs::{
    fake_run_all_domain_benchmark_results, AllDomainFakeRunsReport,
    DEFAULT_ALL_DOMAIN_FAKE_RUN_ROOT,
};
use crate::commands::benchmark::local_essential_pipeline_fake_runs::{
    fake_run_essential_pipelines, EssentialPipelineFakeRunsReport,
    DEFAULT_ESSENTIAL_PIPELINE_FAKE_RUN_ROOT,
};
use crate::commands::benchmark::local_real_smoke_core_subset::{
    render_real_smoke_core_subset, RealSmokeCoreSubsetExecutionKind, RealSmokeCoreSubsetReport,
    DEFAULT_REAL_SMOKE_CORE_SUBSET_PATH,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_FULL_BENCHMARK_RESULT_COLLECTOR_PATH: &str =
    "benchmarks/readiness/full-result-collector-test.json";
const FULL_BENCHMARK_RESULT_COLLECTOR_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.full_benchmark_result_collector.v1";

const ESSENTIAL_PIPELINE_NODE_COUNT: usize = 93;
const REAL_SMOKE_ROW_COUNT: usize = 4;
const INSUFFICIENT_DATA_ROW_COUNT: usize = 1;
const UNSUPPORTED_PAIR_ROW_COUNT: usize = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum FullBenchmarkResultSurfaceKind {
    BenchmarkExpected,
    PipelineFakeRun,
    FakeRun,
    FakeFailure,
    MissingResultAudit,
    RealSmoke,
    FailureClassification,
    UnsupportedPair,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum FullBenchmarkResultStatus {
    Expected,
    Succeeded,
    Failed,
    Present,
    MissingResult,
    InsufficientData,
    UnsupportedPair,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct FullBenchmarkResultCollectorRow {
    pub(crate) record_id: String,
    pub(crate) source_surface: String,
    pub(crate) surface_kind: FullBenchmarkResultSurfaceKind,
    pub(crate) result_status: FullBenchmarkResultStatus,
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) pipeline_id: Option<String>,
    pub(crate) node_id: Option<String>,
    pub(crate) result_id: Option<String>,
    pub(crate) execution_id: Option<String>,
    pub(crate) corpus_id: Option<String>,
    pub(crate) asset_profile_id: Option<String>,
    pub(crate) report_section: Option<String>,
    pub(crate) evidence_path: String,
    pub(crate) manifest_path: Option<String>,
    pub(crate) declared_output_count: usize,
    pub(crate) normalized_metric_count: usize,
    pub(crate) detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct FullBenchmarkResultCollectorReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) row_count: usize,
    pub(crate) benchmark_expected_row_count: usize,
    pub(crate) pipeline_fake_run_row_count: usize,
    pub(crate) fake_run_row_count: usize,
    pub(crate) fake_failure_row_count: usize,
    pub(crate) missing_result_audit_row_count: usize,
    pub(crate) real_smoke_row_count: usize,
    pub(crate) insufficient_data_row_count: usize,
    pub(crate) unsupported_pair_row_count: usize,
    pub(crate) missing_result_status_count: usize,
    pub(crate) insufficient_data_status_count: usize,
    pub(crate) unsupported_pair_status_count: usize,
    pub(crate) passes_behavior_test: bool,
    pub(crate) surface_kind_counts: BTreeMap<String, usize>,
    pub(crate) result_status_counts: BTreeMap<String, usize>,
    pub(crate) domain_counts: BTreeMap<String, usize>,
    pub(crate) rows: Vec<FullBenchmarkResultCollectorRow>,
}

pub(crate) fn run_render_full_benchmark_result_collector(
    args: &parse::BenchReadinessRenderFullBenchmarkResultCollectorArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_full_benchmark_result_collector(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_FULL_BENCHMARK_RESULT_COLLECTOR_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_full_benchmark_result_collector(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<FullBenchmarkResultCollectorReport> {
    let absolute_output_path = repo_relative_path(repo_root, &output_path);
    if let Some(parent) = absolute_output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    let expected_report = render_all_domain_expected_benchmark_results(
        repo_root,
        PathBuf::from(DEFAULT_ALL_DOMAIN_EXPECTED_BENCHMARK_RESULTS_PATH),
    )?;
    let pipeline_report = fake_run_essential_pipelines(
        repo_root,
        PathBuf::from(DEFAULT_ESSENTIAL_PIPELINE_FAKE_RUN_ROOT),
    )?;
    let fake_run_report = fake_run_all_domain_benchmark_results(
        repo_root,
        PathBuf::from(DEFAULT_ALL_DOMAIN_FAKE_RUN_ROOT),
    )?;
    let fake_failure_report = fake_run_all_domain_failures(
        repo_root,
        PathBuf::from(DEFAULT_ALL_DOMAIN_FAKE_FAILURE_ROOT),
        7,
    )?;
    let missing_result_report = render_all_domain_missing_result_test(
        repo_root,
        PathBuf::from(DEFAULT_ALL_DOMAIN_MISSING_RESULT_TEST_PATH),
    )?;
    let real_smoke_report = render_real_smoke_core_subset(
        repo_root,
        PathBuf::from(DEFAULT_REAL_SMOKE_CORE_SUBSET_PATH),
    )?;
    let failure_classification_report = render_all_domain_failure_classification(
        repo_root,
        PathBuf::from(DEFAULT_ALL_DOMAIN_FAILURE_CLASSIFICATION_PATH),
    )?;

    let mut rows = Vec::new();
    rows.extend(collect_expected_rows(&expected_report));
    rows.extend(collect_pipeline_rows(repo_root, &pipeline_report)?);
    rows.extend(collect_fake_run_rows(&fake_run_report));
    rows.extend(collect_fake_failure_rows(&fake_failure_report));
    rows.extend(collect_missing_result_rows(&missing_result_report));
    rows.extend(collect_real_smoke_rows(&real_smoke_report));
    rows.extend(collect_insufficient_data_rows(&failure_classification_report)?);
    rows.extend(collect_unsupported_pair_rows(&failure_classification_report)?);
    rows.sort_by(|left, right| {
        left.domain
            .cmp(&right.domain)
            .then_with(|| left.stage_id.cmp(&right.stage_id))
            .then_with(|| left.tool_id.cmp(&right.tool_id))
            .then_with(|| left.record_id.cmp(&right.record_id))
    });

    let mut surface_kind_counts = BTreeMap::<String, usize>::new();
    let mut result_status_counts = BTreeMap::<String, usize>::new();
    let mut domain_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *surface_kind_counts
            .entry(surface_kind_label(row.surface_kind).to_string())
            .or_default() += 1;
        *result_status_counts
            .entry(result_status_label(row.result_status).to_string())
            .or_default() += 1;
        *domain_counts.entry(row.domain.clone()).or_default() += 1;
    }

    let report = FullBenchmarkResultCollectorReport {
        schema_version: FULL_BENCHMARK_RESULT_COLLECTOR_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &absolute_output_path),
        row_count: rows.len(),
        benchmark_expected_row_count: count_surface(
            &surface_kind_counts,
            FullBenchmarkResultSurfaceKind::BenchmarkExpected,
        ),
        pipeline_fake_run_row_count: count_surface(
            &surface_kind_counts,
            FullBenchmarkResultSurfaceKind::PipelineFakeRun,
        ),
        fake_run_row_count: count_surface(
            &surface_kind_counts,
            FullBenchmarkResultSurfaceKind::FakeRun,
        ),
        fake_failure_row_count: count_surface(
            &surface_kind_counts,
            FullBenchmarkResultSurfaceKind::FakeFailure,
        ),
        missing_result_audit_row_count: count_surface(
            &surface_kind_counts,
            FullBenchmarkResultSurfaceKind::MissingResultAudit,
        ),
        real_smoke_row_count: count_surface(
            &surface_kind_counts,
            FullBenchmarkResultSurfaceKind::RealSmoke,
        ),
        insufficient_data_row_count: count_surface(
            &surface_kind_counts,
            FullBenchmarkResultSurfaceKind::FailureClassification,
        ),
        unsupported_pair_row_count: count_surface(
            &surface_kind_counts,
            FullBenchmarkResultSurfaceKind::UnsupportedPair,
        ),
        missing_result_status_count: count_status(
            &result_status_counts,
            FullBenchmarkResultStatus::MissingResult,
        ),
        insufficient_data_status_count: count_status(
            &result_status_counts,
            FullBenchmarkResultStatus::InsufficientData,
        ),
        unsupported_pair_status_count: count_status(
            &result_status_counts,
            FullBenchmarkResultStatus::UnsupportedPair,
        ),
        passes_behavior_test: false,
        surface_kind_counts,
        result_status_counts,
        domain_counts,
        rows,
    };
    let report = ensure_full_benchmark_result_collector_contract(report)?;
    bijux_dna_infra::atomic_write_json(&absolute_output_path, &report)?;
    Ok(report)
}

fn collect_expected_rows(
    report: &AllDomainExpectedBenchmarkResultsReport,
) -> Vec<FullBenchmarkResultCollectorRow> {
    report
        .rows
        .iter()
        .map(|row| FullBenchmarkResultCollectorRow {
            record_id: format!("expected:{}", row.result_id),
            source_surface: "all_domain_expected_benchmark_results".to_string(),
            surface_kind: FullBenchmarkResultSurfaceKind::BenchmarkExpected,
            result_status: FullBenchmarkResultStatus::Expected,
            domain: row.domain.clone(),
            stage_id: row.stage_id.clone(),
            tool_id: row.tool_id.clone(),
            pipeline_id: None,
            node_id: None,
            result_id: Some(row.result_id.clone()),
            execution_id: None,
            corpus_id: Some(row.corpus_id.clone()),
            asset_profile_id: Some(row.asset_profile_id.clone()),
            report_section: Some(row.report_section.clone()),
            evidence_path: report.output_path.clone(),
            manifest_path: None,
            declared_output_count: row.expected_outputs.len(),
            normalized_metric_count: row.expected_metrics.len(),
            detail: "canonical benchmark-ready result binding".to_string(),
        })
        .collect()
}

fn collect_pipeline_rows(
    repo_root: &Path,
    report: &EssentialPipelineFakeRunsReport,
) -> Result<Vec<FullBenchmarkResultCollectorRow>> {
    let mut rows = Vec::with_capacity(report.node_count);
    for pipeline in &report.pipelines {
        for node in &pipeline.nodes {
            let metrics_path = repo_root.join(&node.metrics_path);
            let metrics = read_json_document(&metrics_path)?;
            rows.push(FullBenchmarkResultCollectorRow {
                record_id: format!("pipeline:{}:{}", pipeline.pipeline_id, node.node_id),
                source_surface: "essential_pipeline_fake_runs".to_string(),
                surface_kind: FullBenchmarkResultSurfaceKind::PipelineFakeRun,
                result_status: FullBenchmarkResultStatus::Succeeded,
                domain: node.domain.clone(),
                stage_id: node.stage_id.clone(),
                tool_id: node.tool_id.clone(),
                pipeline_id: Some(pipeline.pipeline_id.clone()),
                node_id: Some(node.node_id.clone()),
                result_id: None,
                execution_id: None,
                corpus_id: json_optional_string_field(&metrics, "corpus_id"),
                asset_profile_id: json_optional_string_field(&metrics, "asset_profile_id"),
                report_section: None,
                evidence_path: node.metrics_path.clone(),
                manifest_path: Some(node.stage_result_path.clone()),
                declared_output_count: node.declared_output_count,
                normalized_metric_count: 0,
                detail: format!(
                    "essential pipeline fake-run node with command source `{}`",
                    node.command_source
                ),
            });
        }
    }
    Ok(rows)
}

fn collect_fake_run_rows(report: &AllDomainFakeRunsReport) -> Vec<FullBenchmarkResultCollectorRow> {
    report
        .results
        .iter()
        .map(|row| FullBenchmarkResultCollectorRow {
            record_id: format!("fake-run:{}", row.result_id),
            source_surface: "all_domain_fake_runs".to_string(),
            surface_kind: FullBenchmarkResultSurfaceKind::FakeRun,
            result_status: FullBenchmarkResultStatus::Succeeded,
            domain: row.domain.clone(),
            stage_id: row.stage_id.clone(),
            tool_id: row.tool_id.clone(),
            pipeline_id: None,
            node_id: None,
            result_id: Some(row.result_id.clone()),
            execution_id: None,
            corpus_id: Some(row.corpus_id.clone()),
            asset_profile_id: Some(row.asset_profile_id.clone()),
            report_section: None,
            evidence_path: row.metrics_path.clone(),
            manifest_path: Some(row.stage_result_path.clone()),
            declared_output_count: row.declared_output_count,
            normalized_metric_count: row.expected_metric_count,
            detail: format!("fake-run materialized via `{}`", row.command_source),
        })
        .collect()
}

fn collect_fake_failure_rows(
    report: &AllDomainFakeFailuresManifest,
) -> Vec<FullBenchmarkResultCollectorRow> {
    report
        .failures
        .iter()
        .map(|row| FullBenchmarkResultCollectorRow {
            record_id: format!("fake-failure:{}", row.result_id),
            source_surface: "all_domain_fake_failures".to_string(),
            surface_kind: FullBenchmarkResultSurfaceKind::FakeFailure,
            result_status: FullBenchmarkResultStatus::Failed,
            domain: row.domain.clone(),
            stage_id: row.stage_id.clone(),
            tool_id: row.tool_id.clone(),
            pipeline_id: None,
            node_id: None,
            result_id: Some(row.result_id.clone()),
            execution_id: None,
            corpus_id: Some(row.corpus_id.clone()),
            asset_profile_id: Some(row.asset_profile_id.clone()),
            report_section: None,
            evidence_path: row.failure_record_path.clone(),
            manifest_path: None,
            declared_output_count: row.failed_output_count,
            normalized_metric_count: 0,
            detail: format!("fake failure recorded exit_code={}", row.exit_code),
        })
        .collect()
}

fn collect_missing_result_rows(
    report: &AllDomainMissingResultTestReport,
) -> Vec<FullBenchmarkResultCollectorRow> {
    report
        .rows
        .iter()
        .map(|row| FullBenchmarkResultCollectorRow {
            record_id: format!("missing-audit:{}", row.result_id),
            source_surface: "all_domain_missing_result_test".to_string(),
            surface_kind: FullBenchmarkResultSurfaceKind::MissingResultAudit,
            result_status: match row.result_status {
                AllDomainMissingResultStatus::Present => FullBenchmarkResultStatus::Present,
                AllDomainMissingResultStatus::MissingResult => {
                    FullBenchmarkResultStatus::MissingResult
                }
            },
            domain: row.domain.clone(),
            stage_id: row.stage_id.clone(),
            tool_id: row.tool_id.clone(),
            pipeline_id: None,
            node_id: None,
            result_id: Some(row.result_id.clone()),
            execution_id: None,
            corpus_id: Some(row.corpus_id.clone()),
            asset_profile_id: Some(row.asset_profile_id.clone()),
            report_section: Some(row.report_section.clone()),
            evidence_path: row.audit_manifest_path.clone(),
            manifest_path: Some(row.audit_manifest_path.clone()),
            declared_output_count: row.expected_output_artifact_ids.len(),
            normalized_metric_count: row.expected_metrics.len(),
            detail: row.reason.clone(),
        })
        .collect()
}

fn collect_real_smoke_rows(
    report: &RealSmokeCoreSubsetReport,
) -> Vec<FullBenchmarkResultCollectorRow> {
    report
        .rows
        .iter()
        .map(|row| FullBenchmarkResultCollectorRow {
            record_id: format!("real-smoke:{}", row.execution_id),
            source_surface: "real_smoke_core_subset".to_string(),
            surface_kind: FullBenchmarkResultSurfaceKind::RealSmoke,
            result_status: FullBenchmarkResultStatus::Succeeded,
            domain: row.domain.clone(),
            stage_id: row.stage_id.clone(),
            tool_id: row.tool_id.clone(),
            pipeline_id: None,
            node_id: None,
            result_id: None,
            execution_id: Some(row.execution_id.clone()),
            corpus_id: Some(row.corpus_id.clone()),
            asset_profile_id: Some(row.asset_profile_id.clone()),
            report_section: None,
            evidence_path: row.evidence_path.clone(),
            manifest_path: row.stage_result_manifest_path.clone(),
            declared_output_count: 0,
            normalized_metric_count: row.normalized_metric_count,
            detail: match row.execution_kind {
                RealSmokeCoreSubsetExecutionKind::Stage => {
                    "governed real-smoke stage execution".to_string()
                }
                RealSmokeCoreSubsetExecutionKind::PipelineBridge => {
                    "governed real-smoke pipeline bridge execution".to_string()
                }
            },
        })
        .collect()
}

fn collect_insufficient_data_rows(
    report: &AllDomainFailureClassificationReport,
) -> Result<Vec<FullBenchmarkResultCollectorRow>> {
    let rows = report
        .rows
        .iter()
        .filter(|row| row.class_id == "insufficient_data")
        .map(|row| FullBenchmarkResultCollectorRow {
            record_id: format!("insufficient-data:{}:{}:{}", row.domain, row.stage_id, row.tool_id),
            source_surface: "all_domain_failure_classification".to_string(),
            surface_kind: FullBenchmarkResultSurfaceKind::FailureClassification,
            result_status: FullBenchmarkResultStatus::InsufficientData,
            domain: row.domain.clone(),
            stage_id: row.stage_id.clone(),
            tool_id: row.tool_id.clone(),
            pipeline_id: None,
            node_id: None,
            result_id: row.result_id.clone(),
            execution_id: None,
            corpus_id: None,
            asset_profile_id: None,
            report_section: None,
            evidence_path: row.evidence_path.clone(),
            manifest_path: None,
            declared_output_count: 0,
            normalized_metric_count: 1,
            detail: row.detail.clone(),
        })
        .collect::<Vec<_>>();
    if rows.len() != INSUFFICIENT_DATA_ROW_COUNT {
        bail!(
            "full benchmark result collector requires exactly one insufficient-data row, found {}",
            rows.len()
        );
    }
    Ok(rows)
}

fn collect_unsupported_pair_rows(
    report: &AllDomainFailureClassificationReport,
) -> Result<Vec<FullBenchmarkResultCollectorRow>> {
    let rows = report
        .rows
        .iter()
        .filter(|row| row.class_id == "unsupported_pair")
        .map(|row| FullBenchmarkResultCollectorRow {
            record_id: format!("unsupported-pair:{}:{}:{}", row.domain, row.stage_id, row.tool_id),
            source_surface: "all_domain_failure_classification".to_string(),
            surface_kind: FullBenchmarkResultSurfaceKind::UnsupportedPair,
            result_status: FullBenchmarkResultStatus::UnsupportedPair,
            domain: row.domain.clone(),
            stage_id: row.stage_id.clone(),
            tool_id: row.tool_id.clone(),
            pipeline_id: None,
            node_id: None,
            result_id: row.result_id.clone(),
            execution_id: None,
            corpus_id: None,
            asset_profile_id: None,
            report_section: None,
            evidence_path: row.evidence_path.clone(),
            manifest_path: None,
            declared_output_count: 0,
            normalized_metric_count: 0,
            detail: row.detail.clone(),
        })
        .collect::<Vec<_>>();
    if rows.len() != UNSUPPORTED_PAIR_ROW_COUNT {
        bail!(
            "full benchmark result collector requires exactly one unsupported-pair row, found {}",
            rows.len()
        );
    }
    Ok(rows)
}

fn ensure_full_benchmark_result_collector_contract(
    mut report: FullBenchmarkResultCollectorReport,
) -> Result<FullBenchmarkResultCollectorReport> {
    let expected_total_row_count = report.benchmark_expected_row_count
        + report.pipeline_fake_run_row_count
        + report.fake_run_row_count
        + report.fake_failure_row_count
        + report.missing_result_audit_row_count
        + report.real_smoke_row_count
        + report.insufficient_data_row_count
        + report.unsupported_pair_row_count;
    if report.row_count != expected_total_row_count {
        return Err(anyhow!(
            "full benchmark result collector must emit the sum of its source surfaces (expected {}, found {})",
            expected_total_row_count,
            report.row_count
        ));
    }
    let unique_record_ids =
        report.rows.iter().map(|row| row.record_id.as_str()).collect::<BTreeSet<_>>();
    if unique_record_ids.len() != report.rows.len() {
        return Err(anyhow!(
            "full benchmark result collector must keep one unique record_id per row"
        ));
    }
    if report.pipeline_fake_run_row_count != ESSENTIAL_PIPELINE_NODE_COUNT
        || report.real_smoke_row_count != REAL_SMOKE_ROW_COUNT
        || report.insufficient_data_row_count != INSUFFICIENT_DATA_ROW_COUNT
        || report.unsupported_pair_row_count != UNSUPPORTED_PAIR_ROW_COUNT
    {
        return Err(anyhow!(
            "full benchmark result collector fixed source-surface counts drifted from the governed pipeline, smoke, or exception slices"
        ));
    }
    if report.benchmark_expected_row_count != report.fake_run_row_count
        || report.benchmark_expected_row_count != report.fake_failure_row_count
        || report.benchmark_expected_row_count != report.missing_result_audit_row_count
    {
        return Err(anyhow!(
            "full benchmark result collector benchmark, fake-run, fake-failure, and missing-audit surfaces must stay aligned on the governed expected-result count"
        ));
    }
    let expected_succeeded_count =
        report.pipeline_fake_run_row_count + report.fake_run_row_count + report.real_smoke_row_count;
    let expected_present_count =
        report.benchmark_expected_row_count.saturating_sub(report.missing_result_status_count);
    if count_status(&report.result_status_counts, FullBenchmarkResultStatus::Expected)
        != report.benchmark_expected_row_count
        || count_status(&report.result_status_counts, FullBenchmarkResultStatus::Succeeded)
            != expected_succeeded_count
        || count_status(&report.result_status_counts, FullBenchmarkResultStatus::Failed)
            != report.fake_failure_row_count
        || count_status(&report.result_status_counts, FullBenchmarkResultStatus::Present)
            != expected_present_count
        || report.missing_result_status_count != 3
        || report.insufficient_data_status_count != 1
        || report.unsupported_pair_status_count != 1
    {
        return Err(anyhow!(
            "full benchmark result collector status counts drifted from the governed behavior slice"
        ));
    }
    if report.domain_counts.len() != 3
        || report.domain_counts.get("fastq").copied().unwrap_or_default() == 0
        || report.domain_counts.get("bam").copied().unwrap_or_default() == 0
        || report.domain_counts.get("vcf").copied().unwrap_or_default() == 0
    {
        return Err(anyhow!(
            "full benchmark result collector must retain non-empty FASTQ, BAM, and VCF coverage"
        ));
    }

    let missing_rows = report
        .rows
        .iter()
        .filter(|row| row.result_status == FullBenchmarkResultStatus::MissingResult)
        .collect::<Vec<_>>();
    if missing_rows.len() != 3 {
        return Err(anyhow!(
            "full benchmark result collector must keep exactly three missing_result rows"
        ));
    }
    if missing_rows
        .iter()
        .any(|row| row.surface_kind != FullBenchmarkResultSurfaceKind::MissingResultAudit)
    {
        return Err(anyhow!(
            "full benchmark result collector must keep missing_result rows on the missing-result audit surface"
        ));
    }
    let unsupported_rows = report
        .rows
        .iter()
        .filter(|row| row.result_status == FullBenchmarkResultStatus::UnsupportedPair)
        .collect::<Vec<_>>();
    let insufficient_rows = report
        .rows
        .iter()
        .filter(|row| row.result_status == FullBenchmarkResultStatus::InsufficientData)
        .collect::<Vec<_>>();
    if insufficient_rows.len() != 1 {
        return Err(anyhow!(
            "full benchmark result collector must keep exactly one insufficient_data row"
        ));
    }
    if insufficient_rows[0].surface_kind != FullBenchmarkResultSurfaceKind::FailureClassification {
        return Err(anyhow!(
            "full benchmark result collector must keep insufficient_data on the failure-classification surface"
        ));
    }
    if unsupported_rows.len() != 1 {
        return Err(anyhow!(
            "full benchmark result collector must keep exactly one unsupported_pair row"
        ));
    }
    if unsupported_rows[0].surface_kind != FullBenchmarkResultSurfaceKind::UnsupportedPair {
        return Err(anyhow!(
            "full benchmark result collector must keep unsupported_pair distinct from missing-result audit rows"
        ));
    }
    if unsupported_rows[0].stage_id != "vcf.filter" || unsupported_rows[0].tool_id != "samtools" {
        return Err(anyhow!(
            "full benchmark result collector unsupported-pair row drifted from governed evidence"
        ));
    }
    if missing_rows.iter().any(|row| {
        row.domain == unsupported_rows[0].domain
            && row.stage_id == unsupported_rows[0].stage_id
            && row.tool_id == unsupported_rows[0].tool_id
    }) {
        return Err(anyhow!(
            "full benchmark result collector must not collapse missing_result and unsupported_pair into the same binding"
        ));
    }

    for row in &report.rows {
        if row.record_id.trim().is_empty()
            || row.source_surface.trim().is_empty()
            || row.domain.trim().is_empty()
            || row.stage_id.trim().is_empty()
            || row.tool_id.trim().is_empty()
            || row.evidence_path.trim().is_empty()
            || row.detail.trim().is_empty()
        {
            return Err(anyhow!(
                "full benchmark result collector rows must keep stable identity, evidence, and detail fields"
            ));
        }
    }

    report.passes_behavior_test = true;
    Ok(report)
}

fn count_surface(
    counts: &BTreeMap<String, usize>,
    surface_kind: FullBenchmarkResultSurfaceKind,
) -> usize {
    counts.get(surface_kind_label(surface_kind)).copied().unwrap_or(0)
}

fn count_status(counts: &BTreeMap<String, usize>, status: FullBenchmarkResultStatus) -> usize {
    counts.get(result_status_label(status)).copied().unwrap_or(0)
}

fn surface_kind_label(surface_kind: FullBenchmarkResultSurfaceKind) -> &'static str {
    match surface_kind {
        FullBenchmarkResultSurfaceKind::BenchmarkExpected => "benchmark_expected",
        FullBenchmarkResultSurfaceKind::PipelineFakeRun => "pipeline_fake_run",
        FullBenchmarkResultSurfaceKind::FakeRun => "fake_run",
        FullBenchmarkResultSurfaceKind::FakeFailure => "fake_failure",
        FullBenchmarkResultSurfaceKind::MissingResultAudit => "missing_result_audit",
        FullBenchmarkResultSurfaceKind::RealSmoke => "real_smoke",
        FullBenchmarkResultSurfaceKind::FailureClassification => "failure_classification",
        FullBenchmarkResultSurfaceKind::UnsupportedPair => "unsupported_pair",
    }
}

fn result_status_label(status: FullBenchmarkResultStatus) -> &'static str {
    match status {
        FullBenchmarkResultStatus::Expected => "expected",
        FullBenchmarkResultStatus::Succeeded => "succeeded",
        FullBenchmarkResultStatus::Failed => "failed",
        FullBenchmarkResultStatus::Present => "present",
        FullBenchmarkResultStatus::MissingResult => "missing_result",
        FullBenchmarkResultStatus::InsufficientData => "insufficient_data",
        FullBenchmarkResultStatus::UnsupportedPair => "unsupported_pair",
    }
}

fn json_optional_string_field(document: &Value, key: &str) -> Option<String> {
    document.get(key).and_then(Value::as_str).map(str::to_string)
}

fn read_json_document(path: &Path) -> Result<Value> {
    let bytes = fs::read(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_slice(&bytes).with_context(|| format!("parse {}", path.display()))
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}

fn repo_relative_path(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}
