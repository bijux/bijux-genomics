use bijux_dna_api::v1::api::bench::{
    project_full_benchmark_observation_rows, FullBenchmarkSourceObservation,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use bijux_dna_api::v1::api::bench::{
    validate_full_benchmark_result_collector, FullBenchmarkResultBinding,
    FullBenchmarkResultCollectorAdmissionContext, FULL_BENCHMARK_RESULT_COLLECTOR_SCHEMA_V1,
};
pub(crate) use bijux_dna_api::v1::api::bench::{
    FullBenchmarkResultCollectorReport, FullBenchmarkResultCollectorRow, FullBenchmarkResultStatus,
    FullBenchmarkResultSurfaceKind,
};
use serde_json::Value;

use super::all_domain_expected_benchmark_results::{
    render_all_domain_expected_benchmark_results,
    DEFAULT_ALL_DOMAIN_EXPECTED_BENCHMARK_RESULTS_PATH,
};
use super::all_domain_failure_classification::{
    render_all_domain_failure_classification, DEFAULT_ALL_DOMAIN_FAILURE_CLASSIFICATION_PATH,
};
use super::all_domain_missing_result_audit::{
    render_all_domain_missing_result_audit, AllDomainMissingResultStatus,
    AllDomainMissingResultTestReport, DEFAULT_ALL_DOMAIN_MISSING_RESULT_TEST_PATH,
};
use crate::commands::benchmark::local_all_domain_fake_failures::{
    fake_run_all_domain_failures, AllDomainFakeFailuresManifest,
    DEFAULT_ALL_DOMAIN_FAKE_FAILURE_ROOT,
};
use crate::commands::benchmark::local_all_domain_fake_runs::{
    fake_run_all_domain_benchmark_results, DEFAULT_ALL_DOMAIN_FAKE_RUN_ROOT,
};
use crate::commands::benchmark::local_essential_pipeline_fake_runs::{
    fake_run_essential_pipelines, EssentialPipelineFakeRunsReport,
    DEFAULT_ESSENTIAL_PIPELINE_FAKE_RUN_ROOT,
};
use crate::commands::benchmark::local_real_smoke_core_subset::{
    render_real_smoke_core_subset, DEFAULT_REAL_SMOKE_CORE_SUBSET_PATH,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_FULL_BENCHMARK_RESULT_COLLECTOR_PATH: &str =
    "artifacts/benchmark-smoke/readiness/full-result-collector-test.json";
const ESSENTIAL_PIPELINE_NODE_COUNT: usize = 93;
const REAL_SMOKE_ROW_COUNT: usize = 4;
const INSUFFICIENT_DATA_ROW_COUNT: usize = 1;

pub(crate) fn run_render_full_benchmark_result_collector(
    args: &parse::BenchReadinessRenderFullBenchmarkResultCollectorArgs,
) -> Result<()> {
    let repo_root = crate::commands::support::workspace_root::resolve_repo_root()?;
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
    let missing_result_report = render_all_domain_missing_result_audit(
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
    rows.extend(project_full_benchmark_observation_rows(
        FullBenchmarkSourceObservation::Expected(&expected_report),
    )?);
    rows.extend(collect_pipeline_rows(repo_root, &pipeline_report)?);
    rows.extend(project_full_benchmark_observation_rows(
        FullBenchmarkSourceObservation::Simulated(&fake_run_report),
    )?);
    rows.extend(collect_fake_failure_rows(&fake_failure_report));
    rows.extend(collect_missing_result_rows(&missing_result_report));
    rows.extend(project_full_benchmark_observation_rows(
        FullBenchmarkSourceObservation::SmokeSubset(&real_smoke_report),
    )?);
    rows.extend(project_full_benchmark_observation_rows(
        FullBenchmarkSourceObservation::FailureClassification(&failure_classification_report),
    )?);
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
        *surface_kind_counts.entry(row.surface_kind.as_str().to_string()).or_default() += 1;
        *result_status_counts.entry(row.result_status.as_str().to_string()).or_default() += 1;
        *domain_counts.entry(row.domain.clone()).or_default() += 1;
    }

    let report = FullBenchmarkResultCollectorReport {
        schema_version: FULL_BENCHMARK_RESULT_COLLECTOR_SCHEMA_V1.to_string(),
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
    let mut report = report;
    let context = FullBenchmarkResultCollectorAdmissionContext {
        expected_pipeline_fake_run_row_count: ESSENTIAL_PIPELINE_NODE_COUNT,
        expected_real_smoke_row_count: REAL_SMOKE_ROW_COUNT,
        expected_insufficient_data_row_count: INSUFFICIENT_DATA_ROW_COUNT,
        expected_missing_result_row_count: 3,
        expected_domains: BTreeSet::from([
            "bam".to_string(),
            "fastq".to_string(),
            "vcf".to_string(),
        ]),
        expected_unsupported_bindings: BTreeSet::from([FullBenchmarkResultBinding {
            domain: "vcf".to_string(),
            stage_id: "vcf.filter".to_string(),
            tool_id: "samtools".to_string(),
        }]),
    };
    validate_full_benchmark_result_collector(&mut report, &context)?;
    bijux_dna_infra::atomic_write_json(&absolute_output_path, &report)?;
    Ok(report)
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
            source_surface: "all_domain_missing_result_audit".to_string(),
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

fn count_surface(
    counts: &BTreeMap<String, usize>,
    surface_kind: FullBenchmarkResultSurfaceKind,
) -> usize {
    counts.get(surface_kind.as_str()).copied().unwrap_or(0)
}

fn count_status(counts: &BTreeMap<String, usize>, status: FullBenchmarkResultStatus) -> usize {
    counts.get(status.as_str()).copied().unwrap_or(0)
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
