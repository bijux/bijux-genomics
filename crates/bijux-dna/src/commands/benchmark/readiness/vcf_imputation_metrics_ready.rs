use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use super::vcf_adapter_output_coverage::{
    VcfAdapterOutputCoverageStatus, DEFAULT_VCF_ADAPTER_OUTPUT_COVERAGE_PATH,
};
use super::vcf_expected_benchmark_results::DEFAULT_VCF_EXPECTED_BENCHMARK_RESULTS_PATH;
use super::vcf_parser_fixture_coverage::{
    VcfParserFixtureCoverageStatus, DEFAULT_VCF_PARSER_FIXTURE_COVERAGE_PATH,
};
use super::vcf_rendered_commands::VcfRenderedCommandsReport;
use super::vcf_report_map::DEFAULT_VCF_REPORT_MAP_PATH;
use super::vcf_stage_readiness_support::{
    collect_vcf_stage_readiness_bindings, VcfStageReadinessBinding,
};
use crate::commands::benchmark::local_vcf_imputation_metrics_smoke::{
    run_local_vcf_imputation_metrics_smoke, LocalVcfImputationMetricsSmokeReport,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_VCF_IMPUTATION_METRICS_READY_PATH: &str =
    "benchmarks/readiness/vcf/imputation-metrics-ready.json";
const VCF_IMPUTATION_METRICS_READY_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.vcf_imputation_metrics_ready.v1";
const VCF_IMPUTATION_METRICS_STAGE_ID: &str = "vcf.imputation_metrics";
const REQUIRED_METRIC_NAMES: [&str; 8] = [
    "status",
    "concordance",
    "mean_info_score",
    "r2_available",
    "dosage_r2",
    "low_confidence_sites",
    "masked_truth_sites",
    "missing_quality_fields",
];
const REQUIRED_CONCORDANCE: f64 = 1.0;
const REQUIRED_MEAN_INFO_SCORE: f64 = 0.825;
const REQUIRED_DOSAGE_R2: f64 = 0.775;
const REQUIRED_LOW_CONFIDENCE_SITES: u64 = 1;
const REQUIRED_MASKED_TRUTH_SITES: u64 = 1;
const REQUIRED_STATUS: &str = "complete";
const COVERAGE_STATUS_COMPLETE: &str = "complete";
const COVERAGE_STATUS_INCOMPLETE: &str = "incomplete";
const NO_VALUE: &str = "none";
const FLOAT_TOLERANCE: f64 = 1e-9;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct VcfImputationMetricsReadyRow {
    pub(crate) result_id: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) tool_status: String,
    pub(crate) corpus_id: String,
    pub(crate) asset_profile_id: String,
    pub(crate) adapter_id: String,
    pub(crate) parser_id: String,
    pub(crate) schema_id: String,
    pub(crate) retained_scope_state: String,
    pub(crate) retained_scope_detail: String,
    pub(crate) retained_scope_proof_path: String,
    pub(crate) all_domain_active_row_present: bool,
    pub(crate) all_domain_active_row_proof_path: String,
    pub(crate) command_ready: bool,
    pub(crate) command_source: String,
    pub(crate) command_step_count: usize,
    pub(crate) command_step_ids: Vec<String>,
    pub(crate) primary_executables: Vec<String>,
    pub(crate) command_output_path: String,
    pub(crate) command_argv_output_path: String,
    pub(crate) output_ready: bool,
    pub(crate) output_proof_path: String,
    pub(crate) raw_outputs: Vec<String>,
    pub(crate) normalized_metrics_outputs: Vec<String>,
    pub(crate) manifest_output: String,
    pub(crate) index_outputs: Vec<String>,
    pub(crate) parser_ready: bool,
    pub(crate) parser_proof_path: String,
    pub(crate) parser_fixture_parser_id: String,
    pub(crate) parser_fixture_schema_id: String,
    pub(crate) parser_fixture_path: String,
    pub(crate) expected_result_ready: bool,
    pub(crate) expected_result_proof_path: String,
    pub(crate) expected_outputs: Vec<String>,
    pub(crate) expected_metrics: Vec<String>,
    pub(crate) report_section_id: String,
    pub(crate) report_ready: bool,
    pub(crate) report_map_proof_path: String,
    pub(crate) summary_table_id: String,
    pub(crate) report_metric_columns: Vec<String>,
    pub(crate) smoke_ready: bool,
    pub(crate) smoke_command: String,
    pub(crate) smoke_output_root: String,
    pub(crate) smoke_imputation_metrics_path: String,
    pub(crate) smoke_source_imputation_qc_path: String,
    pub(crate) smoke_source_impute_smoke_metrics_path: String,
    pub(crate) smoke_source_imputation_manifest_path: String,
    pub(crate) smoke_stage_result_manifest_path: String,
    pub(crate) smoke_status: String,
    pub(crate) smoke_concordance: Option<f64>,
    pub(crate) smoke_mean_info_score: f64,
    pub(crate) smoke_r2_available: bool,
    pub(crate) smoke_dosage_r2: Option<f64>,
    pub(crate) smoke_low_confidence_sites: u64,
    pub(crate) smoke_masked_truth_sites: u64,
    pub(crate) smoke_missing_quality_fields: Vec<String>,
    pub(crate) required_metric_names: Vec<String>,
    pub(crate) missing_surfaces: Vec<String>,
    pub(crate) coverage_status: String,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct VcfImputationMetricsReadyReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) retained_row_count: usize,
    pub(crate) active_row_count: usize,
    pub(crate) complete_row_count: usize,
    pub(crate) incomplete_row_count: usize,
    pub(crate) checked_surface_count: usize,
    pub(crate) required_metric_names: Vec<String>,
    pub(crate) tool_status_counts: BTreeMap<String, usize>,
    pub(crate) coverage_status_counts: BTreeMap<String, usize>,
    pub(crate) violation_count: usize,
    pub(crate) ok: bool,
    pub(crate) rows: Vec<VcfImputationMetricsReadyRow>,
    pub(crate) violations: Vec<VcfImputationMetricsReadyRow>,
}

pub(crate) fn run_render_vcf_imputation_metrics_ready(
    args: &parse::BenchReadinessRenderVcfImputationMetricsReadyArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_vcf_imputation_metrics_ready(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_IMPUTATION_METRICS_READY_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_vcf_imputation_metrics_ready(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<VcfImputationMetricsReadyReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let report = build_vcf_imputation_metrics_ready_report(repo_root, &output_path)?;
    bijux_dna_infra::atomic_write_json(&output_path, &report)?;
    if !report.ok {
        return Err(anyhow!(
            "vcf.imputation_metrics active retained callers must stay complete across active-scope, command, output, parser, report, and normalized imputation-quality smoke proof"
        ));
    }
    Ok(report)
}

fn build_vcf_imputation_metrics_ready_report(
    repo_root: &Path,
    output_path: &Path,
) -> Result<VcfImputationMetricsReadyReport> {
    let (command_report, bindings) =
        collect_vcf_stage_readiness_bindings(repo_root, VCF_IMPUTATION_METRICS_STAGE_ID)?;
    let active_bindings = bindings
        .into_iter()
        .filter(|binding| binding.retained_row.scope_state == "active")
        .collect::<Vec<_>>();

    let mut rows = Vec::with_capacity(active_bindings.len());
    for binding in active_bindings {
        let smoke_report =
            run_local_vcf_imputation_metrics_smoke(repo_root, &binding.retained_row.tool_id).ok();
        rows.push(build_vcf_imputation_metrics_ready_row(
            &command_report,
            binding,
            smoke_report.as_ref(),
        ));
    }
    rows.sort_by(|left, right| {
        left.stage_id.cmp(&right.stage_id).then_with(|| left.tool_id.cmp(&right.tool_id))
    });

    let complete_row_count =
        rows.iter().filter(|row| row.coverage_status == COVERAGE_STATUS_COMPLETE).count();
    let incomplete_row_count = rows.len().saturating_sub(complete_row_count);
    let active_row_count = rows.iter().filter(|row| row.all_domain_active_row_present).count();
    let mut tool_status_counts = BTreeMap::<String, usize>::new();
    let mut coverage_status_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *tool_status_counts.entry(row.tool_status.clone()).or_default() += 1;
        *coverage_status_counts.entry(row.coverage_status.clone()).or_default() += 1;
    }
    let violations = rows
        .iter()
        .filter(|row| row.coverage_status != COVERAGE_STATUS_COMPLETE)
        .cloned()
        .collect::<Vec<_>>();

    let report = VcfImputationMetricsReadyReport {
        schema_version: VCF_IMPUTATION_METRICS_READY_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, output_path),
        retained_row_count: rows.len(),
        active_row_count,
        complete_row_count,
        incomplete_row_count,
        checked_surface_count: 8,
        required_metric_names: required_metric_names(),
        tool_status_counts,
        coverage_status_counts,
        violation_count: violations.len(),
        ok: violations.is_empty(),
        rows,
        violations,
    };
    ensure_vcf_imputation_metrics_ready_contract(&report)?;
    Ok(report)
}

fn build_vcf_imputation_metrics_ready_row(
    command_report: &VcfRenderedCommandsReport,
    binding: VcfStageReadinessBinding,
    smoke_report: Option<&LocalVcfImputationMetricsSmokeReport>,
) -> VcfImputationMetricsReadyRow {
    let result_id = binding
        .expected_row
        .as_ref()
        .map_or_else(|| retained_result_id(&binding), expected_result_id);
    let required_metric_names = required_metric_names();
    let mut missing_surfaces = Vec::new();

    let retained_scope_active = binding.retained_row.scope_state == "active";
    if !retained_scope_active {
        missing_surfaces.push("retained_vcf_active_scope".to_string());
    }

    let all_domain_active_row_present = binding.active_row.is_some();
    if !all_domain_active_row_present {
        missing_surfaces.push("all_domain_active_row".to_string());
    }

    let command_ready = binding.command_row.as_ref().is_some_and(|row| {
        row.benchmark_status == "benchmark_ready"
            && !row.command_steps.is_empty()
            && !row.script_commands.is_empty()
            && row.command_steps.iter().all(|step| {
                step.argv.first().is_some_and(|value| !value.trim().is_empty())
                    && !step.command.trim().is_empty()
            })
    });
    if !command_ready {
        missing_surfaces.push("vcf_rendered_commands".to_string());
    }

    let output_ready = binding.output_row.as_ref().is_some_and(|row| {
        row.status == VcfAdapterOutputCoverageStatus::Complete
            && row.benchmark_status == "benchmark_ready"
            && contains_artifact_id(&row.raw_outputs, "imputation_qc_json")
            && contains_artifact_id(&row.raw_outputs, "imputation_manifest_json")
            && contains_artifact_id(&row.normalized_metrics, "imputation_metrics_json")
            && !row.manifest.trim().is_empty()
    });
    if !output_ready {
        missing_surfaces.push("vcf_adapter_output_coverage".to_string());
    }

    let parser_ready = binding.parser_row.as_ref().is_some_and(|row| {
        row.coverage_status == VcfParserFixtureCoverageStatus::Covered
            && !row.parser_id.trim().is_empty()
            && !row.parser_fixture_root_path.trim().is_empty()
            && !row.expected_normalized_path.trim().is_empty()
            && row.raw_fixture_count > 0
            && !row.schema_id.trim().is_empty()
    });
    if !parser_ready {
        missing_surfaces.push("vcf_parser_fixture_coverage".to_string());
    }

    let expected_result_ready = binding.expected_row.as_ref().is_some_and(|row| {
        row.report_section == "imputation"
            && row.expected_outputs.iter().any(|value| value == "imputation_metrics_json")
            && required_metric_names
                .iter()
                .all(|metric| row.expected_metrics.iter().any(|value| value == metric))
    });
    if !expected_result_ready {
        missing_surfaces.push("vcf_expected_benchmark_results".to_string());
    }

    let report_ready = binding.report_row.as_ref().is_some_and(|row| {
        row.section_id == "imputation"
            && row.summary_table == "imputation_metrics"
            && required_metric_names
                .iter()
                .all(|metric| row.metric_columns.iter().any(|value| value == metric))
    });
    if !report_ready {
        missing_surfaces.push("vcf_report_map".to_string());
    }

    let smoke_ready = smoke_report.is_some_and(|report| {
        !report.output_root.trim().is_empty()
            && !report.imputation_metrics_path.trim().is_empty()
            && !report.source_imputation_qc_path.trim().is_empty()
            && !report.source_impute_smoke_metrics_path.trim().is_empty()
            && !report.source_imputation_manifest_path.trim().is_empty()
            && !report.stage_result_manifest_path.trim().is_empty()
            && report.status == REQUIRED_STATUS
            && report
                .concordance
                .is_some_and(|value| (value - REQUIRED_CONCORDANCE).abs() <= FLOAT_TOLERANCE)
            && (report.mean_info_score - REQUIRED_MEAN_INFO_SCORE).abs() <= FLOAT_TOLERANCE
            && report.r2_available
            && report
                .dosage_r2
                .is_some_and(|value| (value - REQUIRED_DOSAGE_R2).abs() <= FLOAT_TOLERANCE)
            && report.low_confidence_sites == REQUIRED_LOW_CONFIDENCE_SITES
            && report.masked_truth_sites == REQUIRED_MASKED_TRUTH_SITES
            && report.missing_quality_fields.is_empty()
    });
    if !smoke_ready {
        missing_surfaces.push("local_vcf_imputation_metrics_smoke".to_string());
    }

    let coverage_status = if missing_surfaces.is_empty() {
        COVERAGE_STATUS_COMPLETE.to_string()
    } else {
        COVERAGE_STATUS_INCOMPLETE.to_string()
    };
    let reason = if missing_surfaces.is_empty() {
        format!(
            "active retained VCF imputation-metrics caller `{}` keeps active scope, command, output, parser, expected-result, report, and normalized imputation-quality smoke proof for `vcf.imputation_metrics`",
            binding.retained_row.tool_id
        )
    } else {
        format!(
            "active retained VCF imputation-metrics caller `{}` is missing: {}",
            binding.retained_row.tool_id,
            missing_surfaces.join(", ")
        )
    };

    VcfImputationMetricsReadyRow {
        result_id,
        stage_id: binding.retained_row.stage_id,
        tool_id: binding.retained_row.tool_id,
        tool_status: binding.retained_row.tool_status,
        corpus_id: binding.retained_row.corpus_id,
        asset_profile_id: binding.retained_row.asset_profile_id,
        adapter_id: binding.retained_row.adapter_id,
        parser_id: binding.retained_row.parser_id,
        schema_id: binding.retained_row.schema_id,
        retained_scope_state: binding.retained_row.scope_state,
        retained_scope_detail: binding.retained_row.scope_detail,
        retained_scope_proof_path: binding.retained_row.scope_proof_path,
        all_domain_active_row_present,
        all_domain_active_row_proof_path:
            "benchmarks/readiness/all-domains/active-stage-tool-matrix.tsv".to_string(),
        command_ready,
        command_source: binding
            .command_row
            .as_ref()
            .map_or_else(no_value_string, |row| row.command_source.clone()),
        command_step_count: binding.command_row.as_ref().map_or(0, |row| row.command_steps.len()),
        command_step_ids: binding
            .command_row
            .as_ref()
            .map(|row| row.command_steps.iter().map(|step| step.step_id.clone()).collect())
            .unwrap_or_default(),
        primary_executables: binding
            .command_row
            .as_ref()
            .map(|row| {
                row.command_steps
                    .iter()
                    .map(|step| step.argv.first().cloned().unwrap_or_else(no_value_string))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default(),
        command_output_path: command_report.output_path.clone(),
        command_argv_output_path: command_report.argv_output_path.clone(),
        output_ready,
        output_proof_path: DEFAULT_VCF_ADAPTER_OUTPUT_COVERAGE_PATH.to_string(),
        raw_outputs: binding
            .output_row
            .as_ref()
            .map(|row| row.raw_outputs.clone())
            .unwrap_or_default(),
        normalized_metrics_outputs: binding
            .output_row
            .as_ref()
            .map(|row| row.normalized_metrics.clone())
            .unwrap_or_default(),
        manifest_output: binding
            .output_row
            .as_ref()
            .map_or_else(no_value_string, |row| row.manifest.clone()),
        index_outputs: binding
            .output_row
            .as_ref()
            .map(|row| row.index_outputs.clone())
            .unwrap_or_default(),
        parser_ready,
        parser_proof_path: DEFAULT_VCF_PARSER_FIXTURE_COVERAGE_PATH.to_string(),
        parser_fixture_parser_id: binding
            .parser_row
            .as_ref()
            .map_or_else(no_value_string, |row| row.parser_fixture_parser_id.clone()),
        parser_fixture_schema_id: binding
            .parser_row
            .as_ref()
            .map_or_else(no_value_string, |row| row.parser_fixture_schema_id.clone()),
        parser_fixture_path: binding
            .parser_row
            .as_ref()
            .map_or_else(no_value_string, |row| row.parser_fixture_root_path.clone()),
        expected_result_ready,
        expected_result_proof_path: DEFAULT_VCF_EXPECTED_BENCHMARK_RESULTS_PATH.to_string(),
        expected_outputs: binding
            .expected_row
            .as_ref()
            .map(|row| row.expected_outputs.clone())
            .unwrap_or_default(),
        expected_metrics: binding
            .expected_row
            .as_ref()
            .map(|row| row.expected_metrics.clone())
            .unwrap_or_default(),
        report_section_id: binding
            .expected_row
            .as_ref()
            .map_or_else(no_value_string, |row| row.report_section.clone()),
        report_ready,
        report_map_proof_path: DEFAULT_VCF_REPORT_MAP_PATH.to_string(),
        summary_table_id: binding
            .report_row
            .as_ref()
            .map_or_else(no_value_string, |row| row.summary_table.clone()),
        report_metric_columns: binding
            .report_row
            .as_ref()
            .map(|row| row.metric_columns.clone())
            .unwrap_or_default(),
        smoke_ready,
        smoke_command: smoke_report.map_or_else(no_value_string, |report| report.command.clone()),
        smoke_output_root: smoke_report
            .map_or_else(no_value_string, |report| report.output_root.clone()),
        smoke_imputation_metrics_path: smoke_report
            .map_or_else(no_value_string, |report| report.imputation_metrics_path.clone()),
        smoke_source_imputation_qc_path: smoke_report
            .map_or_else(no_value_string, |report| report.source_imputation_qc_path.clone()),
        smoke_source_impute_smoke_metrics_path: smoke_report
            .map_or_else(no_value_string, |report| report.source_impute_smoke_metrics_path.clone()),
        smoke_source_imputation_manifest_path: smoke_report
            .map_or_else(no_value_string, |report| report.source_imputation_manifest_path.clone()),
        smoke_stage_result_manifest_path: smoke_report
            .map_or_else(no_value_string, |report| report.stage_result_manifest_path.clone()),
        smoke_status: smoke_report.map_or_else(no_value_string, |report| report.status.clone()),
        smoke_concordance: smoke_report.and_then(|report| report.concordance.map(round_metric)),
        smoke_mean_info_score: smoke_report
            .map_or(0.0, |report| round_metric(report.mean_info_score)),
        smoke_r2_available: smoke_report.is_some_and(|report| report.r2_available),
        smoke_dosage_r2: smoke_report.and_then(|report| report.dosage_r2.map(round_metric)),
        smoke_low_confidence_sites: smoke_report.map_or(0, |report| report.low_confidence_sites),
        smoke_masked_truth_sites: smoke_report.map_or(0, |report| report.masked_truth_sites),
        smoke_missing_quality_fields: smoke_report
            .map(|report| report.missing_quality_fields.clone())
            .unwrap_or_default(),
        required_metric_names,
        missing_surfaces,
        coverage_status,
        reason,
    }
}

fn ensure_vcf_imputation_metrics_ready_contract(
    report: &VcfImputationMetricsReadyReport,
) -> Result<()> {
    if report.retained_row_count != report.rows.len() {
        return Err(anyhow!(
            "VCF imputation-metrics readiness must keep exactly one row per active retained `vcf.imputation_metrics` binding"
        ));
    }
    if report.rows.is_empty() {
        return Err(anyhow!(
            "VCF imputation-metrics readiness must keep at least one active retained caller row"
        ));
    }
    if report.checked_surface_count != 8 {
        return Err(anyhow!(
            "VCF imputation-metrics readiness must record exactly 8 checked surfaces"
        ));
    }
    let unique_results =
        report.rows.iter().map(|row| row.result_id.as_str()).collect::<BTreeSet<_>>().len();
    if unique_results != report.rows.len() {
        return Err(anyhow!(
            "VCF imputation-metrics readiness must keep one unique result_id per active retained caller row"
        ));
    }
    for row in &report.rows {
        if row.stage_id != VCF_IMPUTATION_METRICS_STAGE_ID {
            return Err(anyhow!(
                "VCF imputation-metrics readiness row `{}` drifted away from the `vcf.imputation_metrics` stage",
                row.stage_id
            ));
        }
        if row.coverage_status == COVERAGE_STATUS_COMPLETE && !row.missing_surfaces.is_empty() {
            return Err(anyhow!(
                "VCF imputation-metrics readiness row `{}` / `{}` cannot be complete while listing missing surfaces",
                row.stage_id,
                row.tool_id
            ));
        }
    }
    Ok(())
}

fn expected_result_id(
    row: &super::vcf_expected_benchmark_results::VcfExpectedBenchmarkResultRow,
) -> String {
    format!("vcf:{}:{}:{}:{}", row.corpus_id, row.stage_id, row.asset_profile_id, row.tool_id)
}

fn retained_result_id(binding: &VcfStageReadinessBinding) -> String {
    format!(
        "vcf:{}:{}:{}:{}",
        binding.retained_row.corpus_id,
        binding.retained_row.stage_id,
        binding.retained_row.asset_profile_id,
        binding.retained_row.tool_id
    )
}

fn contains_artifact_id(entries: &[String], expected_id: &str) -> bool {
    entries.iter().any(|entry| artifact_id(entry) == expected_id)
}

fn artifact_id(entry: &str) -> &str {
    entry.split_once('=').map_or(entry, |(id, _)| id)
}

fn required_metric_names() -> Vec<String> {
    REQUIRED_METRIC_NAMES.iter().map(|value| (*value).to_string()).collect()
}

fn no_value_string() -> String {
    NO_VALUE.to_string()
}

fn round_metric(value: f64) -> f64 {
    (value * 1_000_000.0).round() / 1_000_000.0
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

#[cfg(test)]
mod tests {
    use super::{
        build_vcf_imputation_metrics_ready_row, render_vcf_imputation_metrics_ready,
        DEFAULT_VCF_IMPUTATION_METRICS_READY_PATH,
    };
    use crate::commands::benchmark::readiness::vcf_stage_readiness_support::collect_vcf_stage_readiness_bindings;

    fn repo_root() -> std::path::PathBuf {
        let crate_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        crate_root.parent().and_then(std::path::Path::parent).expect("repo root").to_path_buf()
    }

    #[test]
    fn render_vcf_imputation_metrics_ready_tracks_active_beagle_row() {
        let repo_root = repo_root();
        let output_root = tempfile::tempdir().expect("tempdir");
        let report = render_vcf_imputation_metrics_ready(
            &repo_root,
            output_root.path().join(DEFAULT_VCF_IMPUTATION_METRICS_READY_PATH),
        )
        .expect("render imputation metrics readiness");

        assert_eq!(report.schema_version, "bijux.bench.readiness.vcf_imputation_metrics_ready.v1");
        assert_eq!(report.output_path, "benchmarks/readiness/vcf/imputation-metrics-ready.json");
        assert_eq!(report.retained_row_count, 1);
        assert_eq!(report.active_row_count, 1);
        assert_eq!(report.complete_row_count, 1);
        assert_eq!(report.incomplete_row_count, 0);
        assert!(report.ok);

        let row = report.rows.first().expect("imputation metrics row");
        assert_eq!(row.stage_id, "vcf.imputation_metrics");
        assert_eq!(row.tool_id, "beagle");
        assert_eq!(row.coverage_status, "complete");
        assert!(row.expected_metrics.iter().any(|metric| metric == "concordance"));
        assert!(row.expected_metrics.iter().any(|metric| metric == "dosage_r2"));
        assert!(row.report_metric_columns.iter().any(|metric| metric == "concordance"));
        assert!(row.report_metric_columns.iter().any(|metric| metric == "dosage_r2"));
        assert_eq!(row.smoke_concordance, Some(1.0));
        assert_eq!(row.smoke_dosage_r2, Some(0.775));
    }

    #[test]
    fn missing_smoke_surface_marks_imputation_metrics_row_incomplete() {
        let repo_root = repo_root();
        let (command_report, bindings) =
            collect_vcf_stage_readiness_bindings(&repo_root, "vcf.imputation_metrics")
                .expect("collect imputation metrics bindings");
        let binding = bindings
            .into_iter()
            .find(|binding| binding.retained_row.scope_state == "active")
            .expect("active imputation metrics binding");

        let row = build_vcf_imputation_metrics_ready_row(&command_report, binding, None);
        assert_eq!(row.coverage_status, "incomplete");
        assert!(row
            .missing_surfaces
            .iter()
            .any(|surface| surface == "local_vcf_imputation_metrics_smoke"));
    }
}
