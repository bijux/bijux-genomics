use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;
use serde_json::Value;

use super::all_domain_expected_benchmark_results::{
    render_all_domain_expected_benchmark_results, AllDomainExpectedBenchmarkResultsReport,
    DEFAULT_ALL_DOMAIN_EXPECTED_BENCHMARK_RESULTS_PATH,
};
use super::all_domain_failure_classification::{
    render_all_domain_failure_classification, AllDomainFailureClassificationReport,
    AllDomainFailureClassificationRow, DEFAULT_ALL_DOMAIN_FAILURE_CLASSIFICATION_PATH,
};
use super::bam_comparable_metrics::{
    render_bam_comparable_metrics, BamComparableMetricsReport, DEFAULT_BAM_COMPARABLE_METRICS_PATH,
};
use super::essential_pipeline_rendered_commands::collect_essential_pipeline_rendered_command_rows;
use super::fastq_comparable_metrics::{
    render_fastq_comparable_metrics, FastqComparableMetricsReport,
    DEFAULT_FASTQ_COMPARABLE_METRICS_PATH,
};
use super::full_benchmark_result_collector::{
    render_full_benchmark_result_collector, FullBenchmarkResultCollectorReport,
    FullBenchmarkResultStatus, FullBenchmarkResultSurfaceKind,
    DEFAULT_FULL_BENCHMARK_RESULT_COLLECTOR_PATH,
};
use super::stage_tool_resources::{render_stage_tool_resources, DEFAULT_STAGE_TOOL_RESOURCES_PATH};
use super::vcf_comparable_metrics::{
    render_vcf_comparable_metrics, VcfComparableMetricsReport, DEFAULT_VCF_COMPARABLE_METRICS_PATH,
};
use crate::commands::benchmark::local_stage_result_manifest::load_validated_stage_result_manifest_path;
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_FULL_BENCHMARK_REPORT_MARKDOWN_PATH: &str =
    "target/bench-readiness/FASTQ_BAM_VCF_BENCHMARK_REPORT.md";
pub(crate) const DEFAULT_FULL_BENCHMARK_REPORT_JSON_PATH: &str =
    "target/bench-readiness/FASTQ_BAM_VCF_BENCHMARK_REPORT.json";
const FULL_BENCHMARK_REPORT_SCHEMA_VERSION: &str = "bijux.bench.readiness.full_benchmark_report.v1";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct BindingKey {
    domain: String,
    stage_id: String,
    tool_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum FullBenchmarkReportRowStatus {
    Present,
    MissingResult,
    UnsupportedPair,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct FullBenchmarkReportRow {
    pub(crate) report_row_id: String,
    pub(crate) row_status: FullBenchmarkReportRowStatus,
    pub(crate) result_id: Option<String>,
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) corpus_id: String,
    pub(crate) asset_profile_id: String,
    pub(crate) report_section: String,
    pub(crate) pipeline_ids: Vec<String>,
    pub(crate) expected_output_count: usize,
    pub(crate) expected_metric_count: usize,
    pub(crate) comparable_metric_count: usize,
    pub(crate) simulated_elapsed_seconds: Option<f64>,
    pub(crate) real_smoke_execution_id: Option<String>,
    pub(crate) real_smoke_elapsed_seconds: Option<f64>,
    pub(crate) declared_memory_mb: Option<f64>,
    pub(crate) declared_cpu_threads: Option<u32>,
    pub(crate) real_smoke_memory_mb: Option<f64>,
    pub(crate) real_smoke_cpu_threads: Option<u32>,
    pub(crate) evidence_path: String,
    pub(crate) manifest_path: Option<String>,
    pub(crate) detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct FullBenchmarkStageSectionRow {
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) row_count: usize,
    pub(crate) present_row_count: usize,
    pub(crate) missing_result_row_count: usize,
    pub(crate) unsupported_pair_row_count: usize,
    pub(crate) tool_ids: Vec<String>,
    pub(crate) corpus_ids: Vec<String>,
    pub(crate) pipeline_ids: Vec<String>,
    pub(crate) report_sections: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct FullBenchmarkToolSectionRow {
    pub(crate) tool_id: String,
    pub(crate) row_count: usize,
    pub(crate) present_row_count: usize,
    pub(crate) missing_result_row_count: usize,
    pub(crate) unsupported_pair_row_count: usize,
    pub(crate) domains: Vec<String>,
    pub(crate) stage_ids: Vec<String>,
    pub(crate) corpus_ids: Vec<String>,
    pub(crate) pipeline_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct FullBenchmarkCorpusSectionRow {
    pub(crate) corpus_id: String,
    pub(crate) row_count: usize,
    pub(crate) present_row_count: usize,
    pub(crate) missing_result_row_count: usize,
    pub(crate) unsupported_pair_row_count: usize,
    pub(crate) domains: Vec<String>,
    pub(crate) stage_ids: Vec<String>,
    pub(crate) tool_ids: Vec<String>,
    pub(crate) pipeline_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct FullBenchmarkPipelineSectionRow {
    pub(crate) pipeline_id: String,
    pub(crate) row_count: usize,
    pub(crate) present_row_count: usize,
    pub(crate) missing_result_row_count: usize,
    pub(crate) unsupported_pair_row_count: usize,
    pub(crate) domains: Vec<String>,
    pub(crate) stage_ids: Vec<String>,
    pub(crate) tool_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct FullBenchmarkPipelineSection {
    pub(crate) row_count: usize,
    pub(crate) unmapped_row_count: usize,
    pub(crate) rows: Vec<FullBenchmarkPipelineSectionRow>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct FullBenchmarkRuntimeRow {
    pub(crate) report_row_id: String,
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) row_status: FullBenchmarkReportRowStatus,
    pub(crate) simulated_elapsed_seconds: Option<f64>,
    pub(crate) real_smoke_execution_id: Option<String>,
    pub(crate) real_smoke_elapsed_seconds: Option<f64>,
    pub(crate) runtime_source: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct FullBenchmarkMemoryRow {
    pub(crate) report_row_id: String,
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) row_status: FullBenchmarkReportRowStatus,
    pub(crate) declared_memory_mb: Option<f64>,
    pub(crate) declared_cpu_threads: Option<u32>,
    pub(crate) real_smoke_memory_mb: Option<f64>,
    pub(crate) real_smoke_cpu_threads: Option<u32>,
    pub(crate) memory_source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct FullBenchmarkFailureRow {
    pub(crate) record_id: String,
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) result_id: Option<String>,
    pub(crate) evidence_path: String,
    pub(crate) detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct FullBenchmarkFailuresSection {
    pub(crate) simulated_failure_row_count: usize,
    pub(crate) failure_class_row_count: usize,
    pub(crate) simulated_failure_domain_counts: BTreeMap<String, usize>,
    pub(crate) rows: Vec<FullBenchmarkFailureRow>,
    pub(crate) classification_rows: Vec<AllDomainFailureClassificationRow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct FullBenchmarkMissingResultRow {
    pub(crate) result_id: String,
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) corpus_id: String,
    pub(crate) asset_profile_id: String,
    pub(crate) report_section: String,
    pub(crate) audit_manifest_path: Option<String>,
    pub(crate) evidence_path: String,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct FullBenchmarkComparableMetricRow {
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) metric_id: String,
    pub(crate) metric_name: String,
    pub(crate) unit: Option<String>,
    pub(crate) direction: Option<String>,
    pub(crate) required: Option<bool>,
    pub(crate) tool_ids: Vec<String>,
    pub(crate) contract_status: String,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct FullBenchmarkUnsupportedPairRow {
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) evidence_path: String,
    pub(crate) detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct FullBenchmarkReport {
    pub(crate) schema_version: &'static str,
    pub(crate) markdown_output_path: String,
    pub(crate) json_output_path: String,
    pub(crate) row_count: usize,
    pub(crate) expected_result_row_count: usize,
    pub(crate) explicit_unsupported_row_count: usize,
    pub(crate) present_row_count: usize,
    pub(crate) missing_result_row_count: usize,
    pub(crate) unsupported_pair_row_count: usize,
    pub(crate) stage_centric_row_count: usize,
    pub(crate) tool_centric_row_count: usize,
    pub(crate) corpus_centric_row_count: usize,
    pub(crate) pipeline_centric_row_count: usize,
    pub(crate) runtime_row_count: usize,
    pub(crate) memory_row_count: usize,
    pub(crate) comparable_metric_row_count: usize,
    pub(crate) failure_row_count: usize,
    pub(crate) failure_class_row_count: usize,
    pub(crate) passes_behavior_test: bool,
    pub(crate) domain_counts: BTreeMap<String, usize>,
    pub(crate) row_status_counts: BTreeMap<String, usize>,
    pub(crate) rows: Vec<FullBenchmarkReportRow>,
    pub(crate) stage_centric: Vec<FullBenchmarkStageSectionRow>,
    pub(crate) tool_centric: Vec<FullBenchmarkToolSectionRow>,
    pub(crate) corpus_centric: Vec<FullBenchmarkCorpusSectionRow>,
    pub(crate) pipeline_centric: FullBenchmarkPipelineSection,
    pub(crate) runtime: Vec<FullBenchmarkRuntimeRow>,
    pub(crate) memory: Vec<FullBenchmarkMemoryRow>,
    pub(crate) failures: FullBenchmarkFailuresSection,
    pub(crate) missing_results: Vec<FullBenchmarkMissingResultRow>,
    pub(crate) comparable_metrics: Vec<FullBenchmarkComparableMetricRow>,
    pub(crate) unsupported_pairs: Vec<FullBenchmarkUnsupportedPairRow>,
}

#[derive(Debug, Clone)]
struct FakeRunRuntimeEvidence {
    simulated_elapsed_seconds: f64,
}

#[derive(Debug, Clone)]
struct RealSmokeRuntimeEvidence {
    execution_id: String,
    elapsed_seconds: Option<f64>,
    memory_mb: Option<f64>,
    cpu_threads: Option<u32>,
}

pub(crate) fn run_render_full_benchmark_report(
    args: &parse::BenchReadinessRenderFullBenchmarkReportArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_full_benchmark_report(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_FULL_BENCHMARK_REPORT_MARKDOWN_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.markdown_output_path);
    }
    Ok(())
}

pub(crate) fn render_full_benchmark_report(
    repo_root: &Path,
    markdown_output_path: PathBuf,
) -> Result<FullBenchmarkReport> {
    let markdown_output_path = repo_relative_path(repo_root, &markdown_output_path);
    let json_output_path = derive_json_output_path(&markdown_output_path);

    let expected_report = render_all_domain_expected_benchmark_results(
        repo_root,
        PathBuf::from(DEFAULT_ALL_DOMAIN_EXPECTED_BENCHMARK_RESULTS_PATH),
    )?;
    let collector_report = render_full_benchmark_result_collector(
        repo_root,
        PathBuf::from(DEFAULT_FULL_BENCHMARK_RESULT_COLLECTOR_PATH),
    )?;
    let failure_classification_report = render_all_domain_failure_classification(
        repo_root,
        PathBuf::from(DEFAULT_ALL_DOMAIN_FAILURE_CLASSIFICATION_PATH),
    )?;
    let resource_report =
        render_stage_tool_resources(repo_root, PathBuf::from(DEFAULT_STAGE_TOOL_RESOURCES_PATH))?;
    let fastq_comparable_report = render_fastq_comparable_metrics(
        repo_root,
        PathBuf::from(DEFAULT_FASTQ_COMPARABLE_METRICS_PATH),
    )?;
    let bam_comparable_report = render_bam_comparable_metrics(
        repo_root,
        PathBuf::from(DEFAULT_BAM_COMPARABLE_METRICS_PATH),
    )?;
    let vcf_comparable_report = render_vcf_comparable_metrics(
        repo_root,
        PathBuf::from(DEFAULT_VCF_COMPARABLE_METRICS_PATH),
    )?;

    let fake_run_runtime_by_result_id =
        collect_fake_run_runtime_by_result_id(repo_root, &collector_report)?;
    let real_smoke_by_binding =
        collect_real_smoke_runtime_by_binding(repo_root, &collector_report)?;
    let missing_audit_by_result_id = collector_report
        .rows
        .iter()
        .filter(|row| row.surface_kind == FullBenchmarkResultSurfaceKind::MissingResultAudit)
        .map(|row| {
            let result_id = row.result_id.as_ref().ok_or_else(|| {
                anyhow!("full benchmark report missing-result audit row is missing result_id")
            })?;
            Ok((result_id.clone(), row.clone()))
        })
        .collect::<Result<BTreeMap<_, _>>>()?;
    let unsupported_rows = collector_report
        .rows
        .iter()
        .filter(|row| row.surface_kind == FullBenchmarkResultSurfaceKind::UnsupportedPair)
        .cloned()
        .collect::<Vec<_>>();
    let resource_by_binding = resource_report
        .rows
        .iter()
        .map(|row| (binding_key(&row.domain, &row.stage_id, &row.tool_id), row.clone()))
        .collect::<BTreeMap<_, _>>();
    let pipeline_ids_by_binding = collect_pipeline_ids_by_binding(repo_root)?;
    let comparable_metrics = collect_full_benchmark_comparable_metrics(
        &fastq_comparable_report,
        &bam_comparable_report,
        &vcf_comparable_report,
    );
    let comparable_metric_counts_by_stage = comparable_metrics.iter().fold(
        BTreeMap::<(String, String), usize>::new(),
        |mut acc, row| {
            *acc.entry((row.domain.clone(), row.stage_id.clone())).or_default() += 1;
            acc
        },
    );

    let mut rows = Vec::with_capacity(expected_report.row_count + unsupported_rows.len());
    for expected in &expected_report.rows {
        let missing_audit_row =
            missing_audit_by_result_id.get(&expected.result_id).ok_or_else(|| {
                anyhow!(
                    "full benchmark report is missing missing-result audit coverage for `{}`",
                    expected.result_id
                )
            })?;
        let fake_run_runtime =
            fake_run_runtime_by_result_id.get(&expected.result_id).ok_or_else(|| {
                anyhow!(
                    "full benchmark report is missing fake-run runtime coverage for `{}`",
                    expected.result_id
                )
            })?;
        let binding = binding_key(&expected.domain, &expected.stage_id, &expected.tool_id);
        let pipeline_ids = pipeline_ids_by_binding
            .get(&binding)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .collect::<Vec<_>>();
        let resource = resource_by_binding.get(&binding);
        let real_smoke = real_smoke_by_binding.get(&binding);
        rows.push(FullBenchmarkReportRow {
            report_row_id: expected.result_id.clone(),
            row_status: match missing_audit_row.result_status {
                FullBenchmarkResultStatus::Present => FullBenchmarkReportRowStatus::Present,
                FullBenchmarkResultStatus::MissingResult => {
                    FullBenchmarkReportRowStatus::MissingResult
                }
                other => {
                    return Err(anyhow!(
                        "full benchmark report encountered unexpected missing-result audit status `{}` for `{}`",
                        full_benchmark_result_status_label(other),
                        expected.result_id
                    ))
                }
            },
            result_id: Some(expected.result_id.clone()),
            domain: expected.domain.clone(),
            stage_id: expected.stage_id.clone(),
            tool_id: expected.tool_id.clone(),
            corpus_id: expected.corpus_id.clone(),
            asset_profile_id: expected.asset_profile_id.clone(),
            report_section: expected.report_section.clone(),
            pipeline_ids,
            expected_output_count: expected.expected_outputs.len(),
            expected_metric_count: expected.expected_metrics.len(),
            comparable_metric_count: comparable_metric_counts_by_stage
                .get(&(expected.domain.clone(), expected.stage_id.clone()))
                .copied()
                .unwrap_or_default(),
            simulated_elapsed_seconds: Some(fake_run_runtime.simulated_elapsed_seconds),
            real_smoke_execution_id: real_smoke.as_ref().map(|row| row.execution_id.clone()),
            real_smoke_elapsed_seconds: real_smoke.as_ref().and_then(|row| row.elapsed_seconds),
            declared_memory_mb: resource.map(|row| f64::from(row.memory_gb) * 1024.0),
            declared_cpu_threads: resource.map(|row| row.threads),
            real_smoke_memory_mb: real_smoke.as_ref().and_then(|row| row.memory_mb),
            real_smoke_cpu_threads: real_smoke.as_ref().and_then(|row| row.cpu_threads),
            evidence_path: missing_audit_row.evidence_path.clone(),
            manifest_path: missing_audit_row.manifest_path.clone(),
            detail: missing_audit_row.detail.clone(),
        });
    }

    for row in unsupported_rows {
        let binding = binding_key(&row.domain, &row.stage_id, &row.tool_id);
        rows.push(FullBenchmarkReportRow {
            report_row_id: format!("unsupported:{}:{}:{}", row.domain, row.stage_id, row.tool_id),
            row_status: FullBenchmarkReportRowStatus::UnsupportedPair,
            result_id: None,
            domain: row.domain.clone(),
            stage_id: row.stage_id.clone(),
            tool_id: row.tool_id.clone(),
            corpus_id: "not_applicable".to_string(),
            asset_profile_id: "not_applicable".to_string(),
            report_section: "unsupported_pairs".to_string(),
            pipeline_ids: pipeline_ids_by_binding
                .get(&binding)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .collect(),
            expected_output_count: 0,
            expected_metric_count: 0,
            comparable_metric_count: comparable_metric_counts_by_stage
                .get(&(row.domain.clone(), row.stage_id.clone()))
                .copied()
                .unwrap_or_default(),
            simulated_elapsed_seconds: None,
            real_smoke_execution_id: None,
            real_smoke_elapsed_seconds: None,
            declared_memory_mb: None,
            declared_cpu_threads: None,
            real_smoke_memory_mb: None,
            real_smoke_cpu_threads: None,
            evidence_path: row.evidence_path.clone(),
            manifest_path: row.manifest_path.clone(),
            detail: row.detail.clone(),
        });
    }
    rows.sort_by(|left, right| {
        left.domain
            .cmp(&right.domain)
            .then_with(|| left.stage_id.cmp(&right.stage_id))
            .then_with(|| left.tool_id.cmp(&right.tool_id))
            .then_with(|| left.report_row_id.cmp(&right.report_row_id))
    });

    let stage_centric = build_stage_centric_rows(&rows);
    let tool_centric = build_tool_centric_rows(&rows);
    let corpus_centric = build_corpus_centric_rows(&rows);
    let pipeline_centric = build_pipeline_centric_rows(&rows);
    let runtime = build_runtime_rows(&rows);
    let memory = build_memory_rows(&rows);
    let failures = build_failures_section(&collector_report, &failure_classification_report)?;
    let missing_results = build_missing_result_rows(&rows);
    let unsupported_pairs = build_unsupported_pair_rows(&rows);

    let mut domain_counts = BTreeMap::<String, usize>::new();
    let mut row_status_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *domain_counts.entry(row.domain.clone()).or_default() += 1;
        *row_status_counts
            .entry(report_row_status_label(row.row_status).to_string())
            .or_default() += 1;
    }

    let report = FullBenchmarkReport {
        schema_version: FULL_BENCHMARK_REPORT_SCHEMA_VERSION,
        markdown_output_path: path_relative_to_repo(repo_root, &markdown_output_path),
        json_output_path: path_relative_to_repo(repo_root, &json_output_path),
        row_count: rows.len(),
        expected_result_row_count: expected_report.row_count,
        explicit_unsupported_row_count: unsupported_pairs.len(),
        present_row_count: rows
            .iter()
            .filter(|row| row.row_status == FullBenchmarkReportRowStatus::Present)
            .count(),
        missing_result_row_count: rows
            .iter()
            .filter(|row| row.row_status == FullBenchmarkReportRowStatus::MissingResult)
            .count(),
        unsupported_pair_row_count: rows
            .iter()
            .filter(|row| row.row_status == FullBenchmarkReportRowStatus::UnsupportedPair)
            .count(),
        stage_centric_row_count: stage_centric.len(),
        tool_centric_row_count: tool_centric.len(),
        corpus_centric_row_count: corpus_centric.len(),
        pipeline_centric_row_count: pipeline_centric.rows.len(),
        runtime_row_count: runtime.len(),
        memory_row_count: memory.len(),
        comparable_metric_row_count: comparable_metrics.len(),
        failure_row_count: failures.rows.len(),
        failure_class_row_count: failures.classification_rows.len(),
        passes_behavior_test: false,
        domain_counts,
        row_status_counts,
        rows,
        stage_centric,
        tool_centric,
        corpus_centric,
        pipeline_centric,
        runtime,
        memory,
        failures,
        missing_results,
        comparable_metrics,
        unsupported_pairs,
    };
    let report = ensure_full_benchmark_report_contract(
        report,
        &expected_report,
        &collector_report,
        &failure_classification_report,
    )?;

    if let Some(parent) = markdown_output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&markdown_output_path, render_full_benchmark_report_markdown(&report))
        .with_context(|| format!("write {}", markdown_output_path.display()))?;
    bijux_dna_infra::atomic_write_json(&json_output_path, &report)?;
    Ok(report)
}

fn collect_pipeline_ids_by_binding(
    repo_root: &Path,
) -> Result<BTreeMap<BindingKey, BTreeSet<String>>> {
    let mut pipeline_ids_by_binding = BTreeMap::<BindingKey, BTreeSet<String>>::new();
    for row in collect_essential_pipeline_rendered_command_rows(repo_root)? {
        pipeline_ids_by_binding
            .entry(binding_key(&row.domain, &row.stage_id, &row.tool_id))
            .or_default()
            .insert(row.pipeline_id);
    }
    Ok(pipeline_ids_by_binding)
}

fn collect_fake_run_runtime_by_result_id(
    repo_root: &Path,
    collector_report: &FullBenchmarkResultCollectorReport,
) -> Result<BTreeMap<String, FakeRunRuntimeEvidence>> {
    let mut rows = BTreeMap::<String, FakeRunRuntimeEvidence>::new();
    for row in collector_report
        .rows
        .iter()
        .filter(|row| row.surface_kind == FullBenchmarkResultSurfaceKind::FakeRun)
    {
        let result_id = row.result_id.as_ref().ok_or_else(|| {
            anyhow!("full benchmark report fake-run collector row is missing result_id")
        })?;
        let metrics_path = repo_root.join(&row.evidence_path);
        let payload = read_json_document(&metrics_path)?;
        let simulated_elapsed_seconds = json_f64_field(&payload, "simulated_elapsed_seconds")
            .with_context(|| {
                format!("read simulated elapsed seconds from {}", metrics_path.display())
            })?;
        rows.insert(result_id.clone(), FakeRunRuntimeEvidence { simulated_elapsed_seconds });
    }
    Ok(rows)
}

fn collect_real_smoke_runtime_by_binding(
    repo_root: &Path,
    collector_report: &FullBenchmarkResultCollectorReport,
) -> Result<BTreeMap<BindingKey, RealSmokeRuntimeEvidence>> {
    let mut rows = BTreeMap::<BindingKey, RealSmokeRuntimeEvidence>::new();
    for row in collector_report
        .rows
        .iter()
        .filter(|row| row.surface_kind == FullBenchmarkResultSurfaceKind::RealSmoke)
    {
        let manifest = row
            .manifest_path
            .as_ref()
            .map(|manifest_path| {
                load_validated_stage_result_manifest_path(&repo_root.join(manifest_path))
            })
            .transpose()?;
        rows.insert(
            binding_key(&row.domain, &row.stage_id, &row.tool_id),
            RealSmokeRuntimeEvidence {
                execution_id: row.record_id.clone(),
                elapsed_seconds: manifest.as_ref().map(|manifest| manifest.runtime.elapsed_seconds),
                memory_mb: manifest
                    .as_ref()
                    .and_then(|manifest| manifest.resource_metrics.memory_mb),
                cpu_threads: manifest
                    .as_ref()
                    .and_then(|manifest| manifest.resource_metrics.cpu_threads),
            },
        );
    }
    Ok(rows)
}

fn build_stage_centric_rows(rows: &[FullBenchmarkReportRow]) -> Vec<FullBenchmarkStageSectionRow> {
    let mut grouped = BTreeMap::<(String, String), Vec<&FullBenchmarkReportRow>>::new();
    for row in rows {
        grouped.entry((row.domain.clone(), row.stage_id.clone())).or_default().push(row);
    }

    grouped
        .into_iter()
        .map(|((domain, stage_id), grouped_rows)| FullBenchmarkStageSectionRow {
            domain,
            stage_id,
            row_count: grouped_rows.len(),
            present_row_count: grouped_rows
                .iter()
                .filter(|row| row.row_status == FullBenchmarkReportRowStatus::Present)
                .count(),
            missing_result_row_count: grouped_rows
                .iter()
                .filter(|row| row.row_status == FullBenchmarkReportRowStatus::MissingResult)
                .count(),
            unsupported_pair_row_count: grouped_rows
                .iter()
                .filter(|row| row.row_status == FullBenchmarkReportRowStatus::UnsupportedPair)
                .count(),
            tool_ids: grouped_rows
                .iter()
                .map(|row| row.tool_id.clone())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect(),
            corpus_ids: grouped_rows
                .iter()
                .map(|row| row.corpus_id.clone())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect(),
            pipeline_ids: grouped_rows
                .iter()
                .flat_map(|row| row.pipeline_ids.iter().cloned())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect(),
            report_sections: grouped_rows
                .iter()
                .map(|row| row.report_section.clone())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect(),
        })
        .collect()
}

fn build_tool_centric_rows(rows: &[FullBenchmarkReportRow]) -> Vec<FullBenchmarkToolSectionRow> {
    let mut grouped = BTreeMap::<String, Vec<&FullBenchmarkReportRow>>::new();
    for row in rows {
        grouped.entry(row.tool_id.clone()).or_default().push(row);
    }

    grouped
        .into_iter()
        .map(|(tool_id, grouped_rows)| FullBenchmarkToolSectionRow {
            tool_id,
            row_count: grouped_rows.len(),
            present_row_count: grouped_rows
                .iter()
                .filter(|row| row.row_status == FullBenchmarkReportRowStatus::Present)
                .count(),
            missing_result_row_count: grouped_rows
                .iter()
                .filter(|row| row.row_status == FullBenchmarkReportRowStatus::MissingResult)
                .count(),
            unsupported_pair_row_count: grouped_rows
                .iter()
                .filter(|row| row.row_status == FullBenchmarkReportRowStatus::UnsupportedPair)
                .count(),
            domains: grouped_rows
                .iter()
                .map(|row| row.domain.clone())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect(),
            stage_ids: grouped_rows
                .iter()
                .map(|row| row.stage_id.clone())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect(),
            corpus_ids: grouped_rows
                .iter()
                .map(|row| row.corpus_id.clone())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect(),
            pipeline_ids: grouped_rows
                .iter()
                .flat_map(|row| row.pipeline_ids.iter().cloned())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect(),
        })
        .collect()
}

fn build_corpus_centric_rows(
    rows: &[FullBenchmarkReportRow],
) -> Vec<FullBenchmarkCorpusSectionRow> {
    let mut grouped = BTreeMap::<String, Vec<&FullBenchmarkReportRow>>::new();
    for row in rows {
        grouped.entry(row.corpus_id.clone()).or_default().push(row);
    }

    grouped
        .into_iter()
        .map(|(corpus_id, grouped_rows)| FullBenchmarkCorpusSectionRow {
            corpus_id,
            row_count: grouped_rows.len(),
            present_row_count: grouped_rows
                .iter()
                .filter(|row| row.row_status == FullBenchmarkReportRowStatus::Present)
                .count(),
            missing_result_row_count: grouped_rows
                .iter()
                .filter(|row| row.row_status == FullBenchmarkReportRowStatus::MissingResult)
                .count(),
            unsupported_pair_row_count: grouped_rows
                .iter()
                .filter(|row| row.row_status == FullBenchmarkReportRowStatus::UnsupportedPair)
                .count(),
            domains: grouped_rows
                .iter()
                .map(|row| row.domain.clone())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect(),
            stage_ids: grouped_rows
                .iter()
                .map(|row| row.stage_id.clone())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect(),
            tool_ids: grouped_rows
                .iter()
                .map(|row| row.tool_id.clone())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect(),
            pipeline_ids: grouped_rows
                .iter()
                .flat_map(|row| row.pipeline_ids.iter().cloned())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect(),
        })
        .collect()
}

fn build_pipeline_centric_rows(rows: &[FullBenchmarkReportRow]) -> FullBenchmarkPipelineSection {
    let mut grouped = BTreeMap::<String, Vec<&FullBenchmarkReportRow>>::new();
    let mut unmapped_row_count = 0usize;
    for row in rows {
        if row.pipeline_ids.is_empty() {
            unmapped_row_count += 1;
            continue;
        }
        for pipeline_id in &row.pipeline_ids {
            grouped.entry(pipeline_id.clone()).or_default().push(row);
        }
    }

    let rows = grouped
        .into_iter()
        .map(|(pipeline_id, grouped_rows)| FullBenchmarkPipelineSectionRow {
            pipeline_id,
            row_count: grouped_rows.len(),
            present_row_count: grouped_rows
                .iter()
                .filter(|row| row.row_status == FullBenchmarkReportRowStatus::Present)
                .count(),
            missing_result_row_count: grouped_rows
                .iter()
                .filter(|row| row.row_status == FullBenchmarkReportRowStatus::MissingResult)
                .count(),
            unsupported_pair_row_count: grouped_rows
                .iter()
                .filter(|row| row.row_status == FullBenchmarkReportRowStatus::UnsupportedPair)
                .count(),
            domains: grouped_rows
                .iter()
                .map(|row| row.domain.clone())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect(),
            stage_ids: grouped_rows
                .iter()
                .map(|row| row.stage_id.clone())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect(),
            tool_ids: grouped_rows
                .iter()
                .map(|row| row.tool_id.clone())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect(),
        })
        .collect::<Vec<_>>();

    FullBenchmarkPipelineSection { row_count: rows.len(), unmapped_row_count, rows }
}

fn build_runtime_rows(rows: &[FullBenchmarkReportRow]) -> Vec<FullBenchmarkRuntimeRow> {
    rows.iter()
        .map(|row| FullBenchmarkRuntimeRow {
            report_row_id: row.report_row_id.clone(),
            domain: row.domain.clone(),
            stage_id: row.stage_id.clone(),
            tool_id: row.tool_id.clone(),
            row_status: row.row_status,
            simulated_elapsed_seconds: row.simulated_elapsed_seconds,
            real_smoke_execution_id: row.real_smoke_execution_id.clone(),
            real_smoke_elapsed_seconds: row.real_smoke_elapsed_seconds,
            runtime_source: runtime_source_label(row).to_string(),
        })
        .collect()
}

fn build_memory_rows(rows: &[FullBenchmarkReportRow]) -> Vec<FullBenchmarkMemoryRow> {
    rows.iter()
        .map(|row| FullBenchmarkMemoryRow {
            report_row_id: row.report_row_id.clone(),
            domain: row.domain.clone(),
            stage_id: row.stage_id.clone(),
            tool_id: row.tool_id.clone(),
            row_status: row.row_status,
            declared_memory_mb: row.declared_memory_mb,
            declared_cpu_threads: row.declared_cpu_threads,
            real_smoke_memory_mb: row.real_smoke_memory_mb,
            real_smoke_cpu_threads: row.real_smoke_cpu_threads,
            memory_source: memory_source_label(row).to_string(),
        })
        .collect()
}

fn build_failures_section(
    collector_report: &FullBenchmarkResultCollectorReport,
    failure_classification_report: &AllDomainFailureClassificationReport,
) -> Result<FullBenchmarkFailuresSection> {
    let mut simulated_failure_domain_counts = BTreeMap::<String, usize>::new();
    let mut rows = Vec::new();
    for row in collector_report
        .rows
        .iter()
        .filter(|row| row.surface_kind == FullBenchmarkResultSurfaceKind::FakeFailure)
    {
        *simulated_failure_domain_counts.entry(row.domain.clone()).or_default() += 1;
        rows.push(FullBenchmarkFailureRow {
            record_id: row.record_id.clone(),
            domain: row.domain.clone(),
            stage_id: row.stage_id.clone(),
            tool_id: row.tool_id.clone(),
            result_id: row.result_id.clone(),
            evidence_path: row.evidence_path.clone(),
            detail: row.detail.clone(),
        });
    }
    if rows.len() != collector_report.fake_failure_row_count {
        return Err(anyhow!(
            "full benchmark report failure section drifted from governed fake-failure coverage"
        ));
    }

    Ok(FullBenchmarkFailuresSection {
        simulated_failure_row_count: rows.len(),
        failure_class_row_count: failure_classification_report.rows.len(),
        simulated_failure_domain_counts,
        rows,
        classification_rows: failure_classification_report.rows.clone(),
    })
}

fn build_missing_result_rows(
    rows: &[FullBenchmarkReportRow],
) -> Vec<FullBenchmarkMissingResultRow> {
    rows.iter()
        .filter(|row| row.row_status == FullBenchmarkReportRowStatus::MissingResult)
        .map(|row| FullBenchmarkMissingResultRow {
            result_id: row.result_id.clone().expect("missing-result row result_id"),
            domain: row.domain.clone(),
            stage_id: row.stage_id.clone(),
            tool_id: row.tool_id.clone(),
            corpus_id: row.corpus_id.clone(),
            asset_profile_id: row.asset_profile_id.clone(),
            report_section: row.report_section.clone(),
            audit_manifest_path: row.manifest_path.clone(),
            evidence_path: row.evidence_path.clone(),
            reason: row.detail.clone(),
        })
        .collect()
}

fn build_unsupported_pair_rows(
    rows: &[FullBenchmarkReportRow],
) -> Vec<FullBenchmarkUnsupportedPairRow> {
    rows.iter()
        .filter(|row| row.row_status == FullBenchmarkReportRowStatus::UnsupportedPair)
        .map(|row| FullBenchmarkUnsupportedPairRow {
            domain: row.domain.clone(),
            stage_id: row.stage_id.clone(),
            tool_id: row.tool_id.clone(),
            evidence_path: row.evidence_path.clone(),
            detail: row.detail.clone(),
        })
        .collect()
}

fn collect_full_benchmark_comparable_metrics(
    fastq_report: &FastqComparableMetricsReport,
    bam_report: &BamComparableMetricsReport,
    vcf_report: &VcfComparableMetricsReport,
) -> Vec<FullBenchmarkComparableMetricRow> {
    let mut rows = Vec::new();

    for row in &fastq_report.rows {
        for metric in &row.shared_metric_fields {
            rows.push(FullBenchmarkComparableMetricRow {
                domain: "fastq".to_string(),
                stage_id: row.stage_id.clone(),
                metric_id: metric.clone(),
                metric_name: metric.clone(),
                unit: None,
                direction: None,
                required: None,
                tool_ids: row.tool_ids.clone(),
                contract_status: fastq_contract_status_label(row.comparison_contract_status)
                    .to_string(),
                reason: row.reason.clone(),
            });
        }
    }

    for row in &bam_report.rows {
        for metric in &row.shared_metric_fields {
            rows.push(FullBenchmarkComparableMetricRow {
                domain: "bam".to_string(),
                stage_id: row.stage_id.clone(),
                metric_id: metric.clone(),
                metric_name: metric.clone(),
                unit: None,
                direction: None,
                required: None,
                tool_ids: row.tool_ids.clone(),
                contract_status: bam_contract_status_label(row.comparison_contract_status)
                    .to_string(),
                reason: row.reason.clone(),
            });
        }
    }

    for row in &vcf_report.rows {
        rows.push(FullBenchmarkComparableMetricRow {
            domain: "vcf".to_string(),
            stage_id: row.stage_id.clone(),
            metric_id: row.metric_id.clone(),
            metric_name: row.metric_name.clone(),
            unit: Some(row.unit.clone()),
            direction: Some(row.direction.clone()),
            required: Some(row.required),
            tool_ids: row.tools_covered.clone(),
            contract_status: "declared".to_string(),
            reason: format!(
                "governed VCF comparable metric published for `{}` across {} retained tools",
                row.stage_id,
                row.tools_covered.len()
            ),
        });
    }

    rows.sort_by(|left, right| {
        left.domain
            .cmp(&right.domain)
            .then_with(|| left.stage_id.cmp(&right.stage_id))
            .then_with(|| left.metric_id.cmp(&right.metric_id))
    });
    rows
}

fn ensure_full_benchmark_report_contract(
    mut report: FullBenchmarkReport,
    expected_report: &AllDomainExpectedBenchmarkResultsReport,
    collector_report: &FullBenchmarkResultCollectorReport,
    failure_classification_report: &AllDomainFailureClassificationReport,
) -> Result<FullBenchmarkReport> {
    if report.row_count != expected_report.row_count + report.explicit_unsupported_row_count {
        return Err(anyhow!(
            "full benchmark report row count must equal expected-result rows plus explicit unsupported rows"
        ));
    }
    if report.explicit_unsupported_row_count != collector_report.unsupported_pair_row_count {
        return Err(anyhow!(
            "full benchmark report unsupported-pair rows drifted from the governed collector"
        ));
    }
    if report.missing_result_row_count != collector_report.missing_result_status_count {
        return Err(anyhow!(
            "full benchmark report missing-result rows drifted from the governed collector"
        ));
    }
    if report.failure_row_count != collector_report.fake_failure_row_count {
        return Err(anyhow!(
            "full benchmark report failure rows drifted from the governed collector"
        ));
    }
    if report.failure_class_row_count != failure_classification_report.triggered_row_count {
        return Err(anyhow!(
            "full benchmark report failure-class rows drifted from the governed failure classification surface"
        ));
    }
    if report.runtime_row_count != report.row_count || report.memory_row_count != report.row_count {
        return Err(anyhow!(
            "full benchmark report runtime and memory sections must keep one row per report row"
        ));
    }
    let unique_report_row_ids =
        report.rows.iter().map(|row| row.report_row_id.as_str()).collect::<BTreeSet<_>>();
    if unique_report_row_ids.len() != report.rows.len() {
        return Err(anyhow!("full benchmark report must keep a unique report_row_id per row"));
    }
    let expected_result_ids =
        expected_report.rows.iter().map(|row| row.result_id.as_str()).collect::<BTreeSet<_>>();
    let report_result_ids =
        report.rows.iter().filter_map(|row| row.result_id.as_deref()).collect::<BTreeSet<_>>();
    if expected_result_ids != report_result_ids {
        return Err(anyhow!(
            "full benchmark report must keep exactly one canonical report row per expected result_id"
        ));
    }
    for row in &report.rows {
        if row.domain.trim().is_empty()
            || row.stage_id.trim().is_empty()
            || row.tool_id.trim().is_empty()
            || row.corpus_id.trim().is_empty()
            || row.asset_profile_id.trim().is_empty()
            || row.report_section.trim().is_empty()
            || row.evidence_path.trim().is_empty()
        {
            return Err(anyhow!(
                "full benchmark report rows must keep non-empty domain, stage, tool, corpus, asset-profile, section, and evidence fields"
            ));
        }
    }
    if report.missing_results.len() != report.missing_result_row_count {
        return Err(anyhow!(
            "full benchmark report missing-results section drifted from the canonical row slice"
        ));
    }
    if report.unsupported_pairs.len() != report.explicit_unsupported_row_count {
        return Err(anyhow!(
            "full benchmark report unsupported-pairs section drifted from the canonical row slice"
        ));
    }
    if report
        .rows
        .iter()
        .filter(|row| row.row_status == FullBenchmarkReportRowStatus::MissingResult)
        .count()
        == 0
    {
        return Err(anyhow!("full benchmark report must keep missing-result rows visible"));
    }
    if report
        .rows
        .iter()
        .filter(|row| row.row_status == FullBenchmarkReportRowStatus::UnsupportedPair)
        .count()
        == 0
    {
        return Err(anyhow!("full benchmark report must keep unsupported-pair rows visible"));
    }
    report.passes_behavior_test = true;
    Ok(report)
}

fn render_full_benchmark_report_markdown(report: &FullBenchmarkReport) -> String {
    let mut rendered = String::from("# FASTQ + BAM + VCF Benchmark Report\n\n");
    rendered.push_str(&format!(
        "- Report rows: {}\n- Expected-result rows: {}\n- Explicit unsupported rows: {}\n- Present rows: {}\n- Missing-result rows: {}\n- Unsupported-pair rows: {}\n- Failure rows: {}\n- Comparable metric rows: {}\n\n",
        report.row_count,
        report.expected_result_row_count,
        report.explicit_unsupported_row_count,
        report.present_row_count,
        report.missing_result_row_count,
        report.unsupported_pair_row_count,
        report.failure_row_count,
        report.comparable_metric_row_count
    ));

    rendered.push_str("## Stage-Centric\n\n");
    rendered.push_str(
        "| Domain | Stage | Rows | Present | Missing | Unsupported | Tools | Pipelines |\n",
    );
    rendered.push_str("| --- | --- | ---: | ---: | ---: | ---: | --- | --- |\n");
    for row in &report.stage_centric {
        rendered.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} |\n",
            sanitize_markdown_cell(&row.domain),
            sanitize_markdown_cell(&row.stage_id),
            row.row_count,
            row.present_row_count,
            row.missing_result_row_count,
            row.unsupported_pair_row_count,
            sanitize_markdown_cell(&row.tool_ids.join(", ")),
            sanitize_markdown_cell(&row.pipeline_ids.join(", "))
        ));
    }
    rendered.push('\n');

    rendered.push_str("## Tool-Centric\n\n");
    rendered.push_str("| Tool | Rows | Present | Missing | Unsupported | Domains | Stages |\n");
    rendered.push_str("| --- | ---: | ---: | ---: | ---: | --- | --- |\n");
    for row in &report.tool_centric {
        rendered.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} |\n",
            sanitize_markdown_cell(&row.tool_id),
            row.row_count,
            row.present_row_count,
            row.missing_result_row_count,
            row.unsupported_pair_row_count,
            sanitize_markdown_cell(&row.domains.join(", ")),
            sanitize_markdown_cell(&row.stage_ids.join(", "))
        ));
    }
    rendered.push('\n');

    rendered.push_str("## Corpus-Centric\n\n");
    rendered.push_str("| Corpus | Rows | Present | Missing | Unsupported | Domains | Stages |\n");
    rendered.push_str("| --- | ---: | ---: | ---: | ---: | --- | --- |\n");
    for row in &report.corpus_centric {
        rendered.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} |\n",
            sanitize_markdown_cell(&row.corpus_id),
            row.row_count,
            row.present_row_count,
            row.missing_result_row_count,
            row.unsupported_pair_row_count,
            sanitize_markdown_cell(&row.domains.join(", ")),
            sanitize_markdown_cell(&row.stage_ids.join(", "))
        ));
    }
    rendered.push('\n');

    rendered.push_str("## Pipeline-Centric\n\n");
    rendered.push_str(&format!(
        "- Pipeline rows: {}\n- Unmapped report rows: {}\n\n",
        report.pipeline_centric.row_count, report.pipeline_centric.unmapped_row_count
    ));
    rendered.push_str("| Pipeline | Rows | Present | Missing | Unsupported | Domains | Stages |\n");
    rendered.push_str("| --- | ---: | ---: | ---: | ---: | --- | --- |\n");
    for row in &report.pipeline_centric.rows {
        rendered.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} |\n",
            sanitize_markdown_cell(&row.pipeline_id),
            row.row_count,
            row.present_row_count,
            row.missing_result_row_count,
            row.unsupported_pair_row_count,
            sanitize_markdown_cell(&row.domains.join(", ")),
            sanitize_markdown_cell(&row.stage_ids.join(", "))
        ));
    }
    rendered.push('\n');

    rendered.push_str("## Runtime\n\n");
    rendered.push_str("| Report Row | Domain | Stage | Tool | Status | Simulated Elapsed Seconds | Real-Smoke Elapsed Seconds | Source |\n");
    rendered.push_str("| --- | --- | --- | --- | --- | ---: | ---: | --- |\n");
    for row in &report.runtime {
        rendered.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} |\n",
            sanitize_markdown_cell(&row.report_row_id),
            sanitize_markdown_cell(&row.domain),
            sanitize_markdown_cell(&row.stage_id),
            sanitize_markdown_cell(&row.tool_id),
            report_row_status_label(row.row_status),
            format_optional_f64(row.simulated_elapsed_seconds),
            format_optional_f64(row.real_smoke_elapsed_seconds),
            sanitize_markdown_cell(&row.runtime_source)
        ));
    }
    rendered.push('\n');

    rendered.push_str("## Memory\n\n");
    rendered.push_str("| Report Row | Domain | Stage | Tool | Status | Declared Memory MB | Declared CPU Threads | Real-Smoke Memory MB | Real-Smoke CPU Threads | Source |\n");
    rendered.push_str("| --- | --- | --- | --- | --- | ---: | ---: | ---: | ---: | --- |\n");
    for row in &report.memory {
        rendered.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
            sanitize_markdown_cell(&row.report_row_id),
            sanitize_markdown_cell(&row.domain),
            sanitize_markdown_cell(&row.stage_id),
            sanitize_markdown_cell(&row.tool_id),
            report_row_status_label(row.row_status),
            format_optional_f64(row.declared_memory_mb),
            format_optional_u32(row.declared_cpu_threads),
            format_optional_f64(row.real_smoke_memory_mb),
            format_optional_u32(row.real_smoke_cpu_threads),
            sanitize_markdown_cell(&row.memory_source)
        ));
    }
    rendered.push('\n');

    rendered.push_str("## Failures\n\n");
    rendered.push_str(&format!(
        "- Simulated failure rows: {}\n- Failure classification rows: {}\n\n",
        report.failures.simulated_failure_row_count, report.failures.failure_class_row_count
    ));
    rendered
        .push_str("| Failure Class | Domain | Stage | Tool | Source Surface | Status | Detail |\n");
    rendered.push_str("| --- | --- | --- | --- | --- | --- | --- |\n");
    for row in &report.failures.classification_rows {
        rendered.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} |\n",
            sanitize_markdown_cell(&row.class_id),
            sanitize_markdown_cell(&row.domain),
            sanitize_markdown_cell(&row.stage_id),
            sanitize_markdown_cell(&row.tool_id),
            sanitize_markdown_cell(&row.source_surface),
            sanitize_markdown_cell(&row.observed_status),
            sanitize_markdown_cell(&row.detail)
        ));
    }
    rendered.push('\n');

    rendered.push_str("## Missing Results\n\n");
    rendered.push_str(
        "| Result ID | Domain | Stage | Tool | Corpus | Section | Manifest Path | Reason |\n",
    );
    rendered.push_str("| --- | --- | --- | --- | --- | --- | --- | --- |\n");
    for row in &report.missing_results {
        rendered.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} |\n",
            sanitize_markdown_cell(&row.result_id),
            sanitize_markdown_cell(&row.domain),
            sanitize_markdown_cell(&row.stage_id),
            sanitize_markdown_cell(&row.tool_id),
            sanitize_markdown_cell(&row.corpus_id),
            sanitize_markdown_cell(&row.report_section),
            sanitize_markdown_cell(row.audit_manifest_path.as_deref().unwrap_or("")),
            sanitize_markdown_cell(&row.reason)
        ));
    }
    rendered.push('\n');

    rendered.push_str("## Comparable Metrics\n\n");
    rendered.push_str("| Domain | Stage | Metric ID | Metric Name | Unit | Direction | Required | Tools | Contract Status |\n");
    rendered.push_str("| --- | --- | --- | --- | --- | --- | --- | --- | --- |\n");
    for row in &report.comparable_metrics {
        rendered.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
            sanitize_markdown_cell(&row.domain),
            sanitize_markdown_cell(&row.stage_id),
            sanitize_markdown_cell(&row.metric_id),
            sanitize_markdown_cell(&row.metric_name),
            sanitize_markdown_cell(row.unit.as_deref().unwrap_or("")),
            sanitize_markdown_cell(row.direction.as_deref().unwrap_or("")),
            row.required.map(|value| value.to_string()).unwrap_or_default(),
            sanitize_markdown_cell(&row.tool_ids.join(", ")),
            sanitize_markdown_cell(&row.contract_status)
        ));
    }
    rendered.push('\n');

    rendered.push_str("## Unsupported Pairs\n\n");
    rendered.push_str("| Domain | Stage | Tool | Evidence Path | Detail |\n");
    rendered.push_str("| --- | --- | --- | --- | --- |\n");
    for row in &report.unsupported_pairs {
        rendered.push_str(&format!(
            "| {} | {} | {} | {} | {} |\n",
            sanitize_markdown_cell(&row.domain),
            sanitize_markdown_cell(&row.stage_id),
            sanitize_markdown_cell(&row.tool_id),
            sanitize_markdown_cell(&row.evidence_path),
            sanitize_markdown_cell(&row.detail)
        ));
    }

    rendered
}

fn runtime_source_label(row: &FullBenchmarkReportRow) -> &'static str {
    if row.row_status == FullBenchmarkReportRowStatus::UnsupportedPair {
        "not_applicable"
    } else if row.real_smoke_elapsed_seconds.is_some() {
        "fake_run_and_real_smoke"
    } else {
        "fake_run_simulated"
    }
}

fn memory_source_label(row: &FullBenchmarkReportRow) -> &'static str {
    if row.row_status == FullBenchmarkReportRowStatus::UnsupportedPair {
        "not_applicable"
    } else if row.real_smoke_memory_mb.is_some() || row.real_smoke_cpu_threads.is_some() {
        "real_smoke_manifest"
    } else if row.declared_memory_mb.is_some() || row.declared_cpu_threads.is_some() {
        "declared_stage_tool_resource"
    } else {
        "not_available"
    }
}

fn full_benchmark_result_status_label(status: FullBenchmarkResultStatus) -> &'static str {
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

fn report_row_status_label(status: FullBenchmarkReportRowStatus) -> &'static str {
    match status {
        FullBenchmarkReportRowStatus::Present => "present",
        FullBenchmarkReportRowStatus::MissingResult => "missing_result",
        FullBenchmarkReportRowStatus::UnsupportedPair => "unsupported_pair",
    }
}

fn fastq_contract_status_label(
    status: super::fastq_comparable_metrics::FastqComparableMetricContractStatus,
) -> &'static str {
    match status {
        super::fastq_comparable_metrics::FastqComparableMetricContractStatus::Declared => {
            "declared"
        }
        super::fastq_comparable_metrics::FastqComparableMetricContractStatus::MissingSharedMetrics => {
            "missing_shared_metrics"
        }
    }
}

fn bam_contract_status_label(
    status: super::bam_comparable_metrics::BamComparableMetricContractStatus,
) -> &'static str {
    match status {
        super::bam_comparable_metrics::BamComparableMetricContractStatus::Declared => "declared",
        super::bam_comparable_metrics::BamComparableMetricContractStatus::MissingSharedMetrics => {
            "missing_shared_metrics"
        }
    }
}

fn binding_key(domain: &str, stage_id: &str, tool_id: &str) -> BindingKey {
    BindingKey {
        domain: domain.to_string(),
        stage_id: stage_id.to_string(),
        tool_id: tool_id.to_string(),
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

fn read_json_document(path: &Path) -> Result<Value> {
    let payload = fs::read_to_string(path)
        .with_context(|| format!("read JSON document {}", path.display()))?;
    serde_json::from_str(&payload)
        .with_context(|| format!("parse JSON document {}", path.display()))
}

fn json_f64_field(payload: &Value, key: &str) -> Result<f64> {
    payload
        .get(key)
        .and_then(Value::as_f64)
        .ok_or_else(|| anyhow!("JSON document is missing `{key}`"))
}

fn sanitize_markdown_cell(value: &str) -> String {
    value.replace('|', "\\|").replace('\n', " ")
}

fn format_optional_f64(value: Option<f64>) -> String {
    value.map(|value| format!("{value:.3}")).unwrap_or_default()
}

fn format_optional_u32(value: Option<u32>) -> String {
    value.map(|value| value.to_string()).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::{render_full_benchmark_report, DEFAULT_FULL_BENCHMARK_REPORT_MARKDOWN_PATH};
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
    fn full_benchmark_report_tracks_governed_row_counts() {
        let repo_root = repo_root();
        let tempdir = tempfile::tempdir().expect("tempdir");
        let report = render_full_benchmark_report(
            &repo_root,
            tempdir.path().join(DEFAULT_FULL_BENCHMARK_REPORT_MARKDOWN_PATH),
        )
        .expect("render full benchmark report");

        assert_eq!(report.row_count, 121);
        assert_eq!(report.expected_result_row_count, 120);
        assert_eq!(report.explicit_unsupported_row_count, 1);
        assert_eq!(report.present_row_count, 117);
        assert_eq!(report.missing_result_row_count, 3);
        assert_eq!(report.unsupported_pair_row_count, 1);
        assert_eq!(report.failure_row_count, 120);
        assert_eq!(report.failure_class_row_count, 7);
        assert_eq!(report.rows.len(), 121);
        assert_eq!(report.runtime.len(), 121);
        assert_eq!(report.memory.len(), 121);
        assert_eq!(report.missing_results.len(), 3);
        assert_eq!(report.unsupported_pairs.len(), 1);
        assert!(report.passes_behavior_test);
    }

    #[test]
    fn full_benchmark_report_writes_required_sections() {
        let repo_root = repo_root();
        let tempdir = tempfile::tempdir().expect("tempdir");
        let output_path = tempdir.path().join(DEFAULT_FULL_BENCHMARK_REPORT_MARKDOWN_PATH);
        let report =
            render_full_benchmark_report(&repo_root, output_path.clone()).expect("render report");

        let markdown = std::fs::read_to_string(output_path).expect("read markdown");
        assert!(report.markdown_output_path.ends_with(DEFAULT_FULL_BENCHMARK_REPORT_MARKDOWN_PATH));
        assert!(markdown.contains("# FASTQ + BAM + VCF Benchmark Report"));
        assert!(markdown.contains("## Stage-Centric"));
        assert!(markdown.contains("## Tool-Centric"));
        assert!(markdown.contains("## Corpus-Centric"));
        assert!(markdown.contains("## Pipeline-Centric"));
        assert!(markdown.contains("## Runtime"));
        assert!(markdown.contains("## Memory"));
        assert!(markdown.contains("## Failures"));
        assert!(markdown.contains("## Missing Results"));
        assert!(markdown.contains("## Comparable Metrics"));
        assert!(markdown.contains("## Unsupported Pairs"));
        assert!(
            markdown.contains("fastq:corpus-02-edna-mini:fastq.screen_taxonomy:sample-set:kraken2")
        );
        assert!(markdown.contains("vcf.filter"));
        assert!(markdown.contains("samtools"));
    }
}
