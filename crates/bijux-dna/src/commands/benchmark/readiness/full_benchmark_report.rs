use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_api::v1::api::bench::{
    build_full_benchmark_corpus_rows, build_full_benchmark_failures_section,
    build_full_benchmark_memory_rows, build_full_benchmark_missing_result_rows,
    build_full_benchmark_pipeline_rows, build_full_benchmark_runtime_rows,
    build_full_benchmark_stage_rows, build_full_benchmark_tool_rows,
    build_full_benchmark_unsupported_pair_rows, render_full_benchmark_report_markdown,
    validate_full_benchmark_report, FullBenchmarkReportAdmissionContext,
    FULL_BENCHMARK_REPORT_SCHEMA_V1,
};
pub(crate) use bijux_dna_api::v1::api::bench::{
    FullBenchmarkComparableMetricRow, FullBenchmarkReport, FullBenchmarkReportRow,
    FullBenchmarkReportRowStatus,
};
use serde_json::Value;

use super::all_domain_expected_benchmark_results::{
    render_all_domain_expected_benchmark_results,
    DEFAULT_ALL_DOMAIN_EXPECTED_BENCHMARK_RESULTS_PATH,
};
use super::all_domain_failure_classification::{
    render_all_domain_failure_classification, DEFAULT_ALL_DOMAIN_FAILURE_CLASSIFICATION_PATH,
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
use super::scientific_acceptance_thresholds::{
    render_scientific_acceptance_thresholds_from_reports, scientific_acceptance_direction_label,
    scientific_acceptance_insufficiency_behavior_label, scientific_acceptance_pass_rule_label,
    scientific_acceptance_tolerance_kind_label, ScientificAcceptanceThresholdRow,
    DEFAULT_SCIENTIFIC_ACCEPTANCE_THRESHOLDS_PATH,
};
use super::stage_tool_resources::{render_stage_tool_resources, DEFAULT_STAGE_TOOL_RESOURCES_PATH};
use super::vcf_comparable_metrics::{
    render_vcf_comparable_metrics, VcfComparableMetricsReport, DEFAULT_VCF_COMPARABLE_METRICS_PATH,
};
use crate::commands::benchmark::local_stage_result_manifest::load_validated_stage_result_manifest_path;
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_FULL_BENCHMARK_REPORT_MARKDOWN_PATH: &str =
    "artifacts/benchmark-smoke/readiness/all-domains/FASTQ_BAM_VCF_BENCHMARK_REPORT.md";
pub(crate) const DEFAULT_FULL_BENCHMARK_REPORT_JSON_PATH: &str =
    "artifacts/benchmark-smoke/readiness/all-domains/FASTQ_BAM_VCF_BENCHMARK_REPORT.json";
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct BindingKey {
    domain: String,
    stage_id: String,
    tool_id: String,
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
    let repo_root = crate::commands::support::workspace_root::resolve_repo_root()?;
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
    let _cwd_guard = CurrentDirGuard::change_to(repo_root);
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
    let scientific_acceptance_report = render_scientific_acceptance_thresholds_from_reports(
        repo_root,
        PathBuf::from(DEFAULT_SCIENTIFIC_ACCEPTANCE_THRESHOLDS_PATH),
        &fastq_comparable_report,
        &bam_comparable_report,
        &vcf_comparable_report,
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
        &scientific_acceptance_report.rows,
    )?;
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

    let stage_centric = build_full_benchmark_stage_rows(&rows);
    let tool_centric = build_full_benchmark_tool_rows(&rows);
    let corpus_centric = build_full_benchmark_corpus_rows(&rows);
    let pipeline_centric = build_full_benchmark_pipeline_rows(&rows);
    let runtime = build_full_benchmark_runtime_rows(&rows);
    let memory = build_full_benchmark_memory_rows(&rows);
    let failures =
        build_full_benchmark_failures_section(&collector_report, &failure_classification_report)?;
    let missing_results = build_full_benchmark_missing_result_rows(&rows)?;
    let unsupported_pairs = build_full_benchmark_unsupported_pair_rows(&rows);

    let mut domain_counts = BTreeMap::<String, usize>::new();
    let mut row_status_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *domain_counts.entry(row.domain.clone()).or_default() += 1;
        *row_status_counts
            .entry(report_row_status_label(row.row_status).to_string())
            .or_default() += 1;
    }

    let mut report = FullBenchmarkReport {
        schema_version: FULL_BENCHMARK_REPORT_SCHEMA_V1.to_string(),
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
    let admission_context = FullBenchmarkReportAdmissionContext {
        expected_result_ids: expected_report.rows.iter().map(|row| row.result_id.clone()).collect(),
        collector_missing_result_count: collector_report.missing_result_status_count,
        collector_failure_count: collector_report.fake_failure_row_count,
        collector_unsupported_pair_count: collector_report.unsupported_pair_row_count,
        triggered_failure_class_count: failure_classification_report.triggered_row_count,
    };
    validate_full_benchmark_report(&mut report, &admission_context)?;

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
                // Real-smoke wall-clock timings vary across hosts and runs, so governed
                // readiness artifacts keep the stable execution identity without serializing
                // volatile elapsed-seconds values into durable report files.
                elapsed_seconds: None,
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

fn collect_full_benchmark_comparable_metrics(
    fastq_report: &FastqComparableMetricsReport,
    bam_report: &BamComparableMetricsReport,
    vcf_report: &VcfComparableMetricsReport,
    acceptance_rows: &[ScientificAcceptanceThresholdRow],
) -> Result<Vec<FullBenchmarkComparableMetricRow>> {
    let acceptance_by_metric = acceptance_rows_by_metric(acceptance_rows)?;
    let mut rows = Vec::new();

    for row in &fastq_report.rows {
        for metric in &row.shared_metric_fields {
            let acceptance = acceptance_row(&acceptance_by_metric, "fastq", &row.stage_id, metric)?;
            rows.push(FullBenchmarkComparableMetricRow {
                domain: "fastq".to_string(),
                stage_id: row.stage_id.clone(),
                metric_id: metric.clone(),
                metric_name: metric.clone(),
                unit: acceptance.unit.clone(),
                direction: scientific_acceptance_direction_label(acceptance.direction).to_string(),
                tolerance_kind: scientific_acceptance_tolerance_kind_label(
                    acceptance.tolerance_kind,
                )
                .to_string(),
                tolerance_value: acceptance.tolerance_value,
                pass_rule: scientific_acceptance_pass_rule_label(acceptance.pass_rule).to_string(),
                insufficiency_behavior: scientific_acceptance_insufficiency_behavior_label(
                    acceptance.insufficiency_behavior,
                )
                .to_string(),
                required: acceptance.required,
                tool_ids: row.tool_ids.clone(),
                contract_status: fastq_contract_status_label(row.comparison_contract_status)
                    .to_string(),
                reason: row.reason.clone(),
            });
        }
    }

    for row in &bam_report.rows {
        for metric in &row.shared_metric_fields {
            let acceptance = acceptance_row(&acceptance_by_metric, "bam", &row.stage_id, metric)?;
            rows.push(FullBenchmarkComparableMetricRow {
                domain: "bam".to_string(),
                stage_id: row.stage_id.clone(),
                metric_id: metric.clone(),
                metric_name: metric.clone(),
                unit: acceptance.unit.clone(),
                direction: scientific_acceptance_direction_label(acceptance.direction).to_string(),
                tolerance_kind: scientific_acceptance_tolerance_kind_label(
                    acceptance.tolerance_kind,
                )
                .to_string(),
                tolerance_value: acceptance.tolerance_value,
                pass_rule: scientific_acceptance_pass_rule_label(acceptance.pass_rule).to_string(),
                insufficiency_behavior: scientific_acceptance_insufficiency_behavior_label(
                    acceptance.insufficiency_behavior,
                )
                .to_string(),
                required: acceptance.required,
                tool_ids: row.tool_ids.clone(),
                contract_status: bam_contract_status_label(row.comparison_contract_status)
                    .to_string(),
                reason: row.reason.clone(),
            });
        }
    }

    for row in &vcf_report.rows {
        let acceptance =
            acceptance_row(&acceptance_by_metric, "vcf", &row.stage_id, &row.metric_id)?;
        rows.push(FullBenchmarkComparableMetricRow {
            domain: "vcf".to_string(),
            stage_id: row.stage_id.clone(),
            metric_id: row.metric_id.clone(),
            metric_name: row.metric_name.clone(),
            unit: acceptance.unit.clone(),
            direction: scientific_acceptance_direction_label(acceptance.direction).to_string(),
            tolerance_kind: scientific_acceptance_tolerance_kind_label(acceptance.tolerance_kind)
                .to_string(),
            tolerance_value: acceptance.tolerance_value,
            pass_rule: scientific_acceptance_pass_rule_label(acceptance.pass_rule).to_string(),
            insufficiency_behavior: scientific_acceptance_insufficiency_behavior_label(
                acceptance.insufficiency_behavior,
            )
            .to_string(),
            required: acceptance.required,
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
    Ok(rows)
}

fn acceptance_rows_by_metric(
    rows: &[ScientificAcceptanceThresholdRow],
) -> Result<BTreeMap<(String, String, String), ScientificAcceptanceThresholdRow>> {
    let mut by_metric = BTreeMap::new();
    for row in rows {
        let key = (row.domain.clone(), row.stage_id.clone(), row.metric_id.clone());
        if by_metric.insert(key.clone(), row.clone()).is_some() {
            return Err(anyhow!(
                "full benchmark report found duplicate scientific acceptance threshold row for `{}:{}/{}`",
                key.0,
                key.1,
                key.2
            ));
        }
    }
    Ok(by_metric)
}

fn acceptance_row<'a>(
    by_metric: &'a BTreeMap<(String, String, String), ScientificAcceptanceThresholdRow>,
    domain: &str,
    stage_id: &str,
    metric_id: &str,
) -> Result<&'a ScientificAcceptanceThresholdRow> {
    by_metric
        .get(&(domain.to_string(), stage_id.to_string(), metric_id.to_string()))
        .ok_or_else(|| {
            anyhow!(
                "full benchmark report is missing scientific acceptance semantics for `{domain}` / `{stage_id}` / `{metric_id}`"
            )
        })
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

struct CurrentDirGuard {
    original_dir: PathBuf,
}

impl CurrentDirGuard {
    fn change_to(path: &Path) -> Self {
        let original_dir = env::current_dir().expect("capture current dir");
        env::set_current_dir(path).expect("set current dir");
        Self { original_dir }
    }
}

impl Drop for CurrentDirGuard {
    fn drop(&mut self) {
        env::set_current_dir(&self.original_dir).expect("restore current dir");
    }
}

#[cfg(test)]
mod tests {
    use super::{
        render_full_benchmark_report, validate_full_benchmark_report,
        FullBenchmarkReportAdmissionContext, DEFAULT_FULL_BENCHMARK_REPORT_MARKDOWN_PATH,
    };
    use crate::commands::benchmark::repo_locking::acquire_cwd_lock;
    use std::path::PathBuf;

    fn repo_root() -> PathBuf {
        crate::commands::support::workspace_root::resolve_repo_root().expect("repo root")
    }

    #[test]
    fn full_benchmark_report_keeps_consistent_sections() {
        let _cwd_lock = acquire_cwd_lock();
        let repo_root = repo_root();
        let tempdir = tempfile::tempdir_in(repo_root.join("artifacts")).expect("tempdir");
        let report = render_full_benchmark_report(
            &repo_root,
            tempdir.path().join(DEFAULT_FULL_BENCHMARK_REPORT_MARKDOWN_PATH),
        )
        .expect("render full benchmark report");

        assert_eq!(
            report.present_row_count + report.missing_result_row_count,
            report.expected_result_row_count
        );
        assert_eq!(
            report.row_count,
            report.expected_result_row_count + report.unsupported_pair_row_count
        );
        assert_eq!(report.explicit_unsupported_row_count, report.unsupported_pairs.len());
        assert_eq!(report.missing_result_row_count, report.missing_results.len());
        assert_eq!(report.failure_row_count, report.expected_result_row_count);
        assert_eq!(report.failure_class_row_count, report.failures.classification_rows.len());
        assert_eq!(report.rows.len(), report.row_count);
        assert_eq!(report.runtime.len(), report.row_count);
        assert_eq!(report.memory.len(), report.row_count);
        assert!(!report.rows.is_empty(), "full benchmark report should contain benchmark rows");
        assert!(report.comparable_metrics.iter().all(|row| !row.direction.is_empty()));
        assert!(report.comparable_metrics.iter().all(|row| !row.tolerance_kind.is_empty()));
        assert!(report.comparable_metrics.iter().all(|row| !row.pass_rule.is_empty()));
        assert!(report.comparable_metrics.iter().all(|row| !row.insufficiency_behavior.is_empty()));
        assert!(report.passes_behavior_test);

        let context = FullBenchmarkReportAdmissionContext {
            expected_result_ids: report
                .rows
                .iter()
                .filter_map(|row| row.result_id.clone())
                .collect(),
            collector_missing_result_count: report.missing_result_row_count,
            collector_failure_count: report.failure_row_count,
            collector_unsupported_pair_count: report.explicit_unsupported_row_count,
            triggered_failure_class_count: report.failure_class_row_count,
        };
        let mut unsafe_report = report.clone();
        unsafe_report.rows[0].evidence_path = "../outside.json".to_string();
        assert!(validate_full_benchmark_report(&mut unsafe_report, &context).is_err());
    }

    #[test]
    fn full_benchmark_report_writes_required_sections() {
        let _cwd_lock = acquire_cwd_lock();
        let repo_root = repo_root();
        let tempdir = tempfile::tempdir_in(repo_root.join("artifacts")).expect("tempdir");
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
        assert!(markdown.contains("Tolerance Kind"));
        assert!(markdown.contains("Insufficiency Behavior"));
        assert!(markdown.contains("match_reference_structure"));
        assert!(
            markdown.contains("fastq:corpus-02-edna-mini:fastq.screen_taxonomy:sample-set:kraken2")
        );
        assert!(markdown.contains("vcf.filter"));
        assert!(markdown.contains("samtools"));
    }
}
