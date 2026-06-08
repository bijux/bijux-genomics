use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use super::all_domain_active_stage_tool_matrix::{
    collect_all_domain_active_stage_tool_matrix_rows, AllDomainActiveStageToolMatrixRow,
    DEFAULT_ALL_DOMAIN_ACTIVE_STAGE_TOOL_MATRIX_PATH,
};
use super::vcf_active_stage_tool_matrix::{
    collect_vcf_active_stage_tool_matrix_rows, VcfActiveStageToolMatrixRow,
};
use super::vcf_adapter_output_coverage::{
    collect_vcf_adapter_output_coverage_rows, VcfAdapterOutputCoverageRow,
    VcfAdapterOutputCoverageStatus, DEFAULT_VCF_ADAPTER_OUTPUT_COVERAGE_PATH,
};
use super::vcf_expected_benchmark_results::{
    collect_vcf_expected_benchmark_result_rows, VcfExpectedBenchmarkResultRow,
    DEFAULT_VCF_EXPECTED_BENCHMARK_RESULTS_PATH,
};
use super::vcf_parser_coverage::{
    collect_vcf_parser_coverage_rows, VcfParserCoverageRow, VcfParserCoverageStatus,
    DEFAULT_VCF_PARSER_COVERAGE_PATH,
};
use super::vcf_rendered_command_rows::VcfRenderedCommandRow;
use super::vcf_rendered_commands::{
    render_vcf_commands, VcfRenderedCommandsReport, DEFAULT_VCF_RENDERED_COMMANDS_PATH,
};
use super::vcf_report_map::{
    collect_vcf_report_map_rows, VcfReportMapRow, DEFAULT_VCF_REPORT_MAP_PATH,
};
use crate::commands::benchmark::local_vcf_call_smoke::{
    run_local_vcf_call_smoke, LocalVcfCallSmokeReport,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_VCF_CALL_READY_PATH: &str = "benchmarks/readiness/vcf/call-ready.json";
const VCF_CALL_READY_SCHEMA_VERSION: &str = "bijux.bench.readiness.vcf_call_ready.v1";
const VCF_CALL_STAGE_ID: &str = "vcf.call";
const REQUIRED_METRIC_NAMES: [&str; 4] =
    ["variant_count", "snp_count", "indel_count", "sample_count"];
const COVERAGE_STATUS_COMPLETE: &str = "complete";
const COVERAGE_STATUS_INCOMPLETE: &str = "incomplete";
const NO_VALUE: &str = "none";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct BindingKey {
    stage_id: String,
    tool_id: String,
    corpus_id: String,
    asset_profile_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct VcfCallReadyRow {
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
    pub(crate) smoke_output_vcf_path: String,
    pub(crate) smoke_output_tbi_path: String,
    pub(crate) smoke_metrics_path: String,
    pub(crate) smoke_stage_result_manifest_path: String,
    pub(crate) smoke_parseable: bool,
    pub(crate) smoke_gt_present: bool,
    pub(crate) smoke_gl_present: bool,
    pub(crate) smoke_variant_count: u64,
    pub(crate) smoke_snp_count: u64,
    pub(crate) smoke_indel_count: u64,
    pub(crate) smoke_sample_count: u64,
    pub(crate) required_metric_names: Vec<String>,
    pub(crate) missing_surfaces: Vec<String>,
    pub(crate) coverage_status: String,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct VcfCallReadyReport {
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
    pub(crate) rows: Vec<VcfCallReadyRow>,
    pub(crate) violations: Vec<VcfCallReadyRow>,
}

pub(crate) fn run_render_vcf_call_ready(
    args: &parse::BenchReadinessRenderVcfCallReadyArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_vcf_call_ready(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_CALL_READY_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_vcf_call_ready(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<VcfCallReadyReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let report = build_vcf_call_ready_report(repo_root, &output_path)?;
    bijux_dna_infra::atomic_write_json(&output_path, &report)?;
    if !report.ok {
        return Err(anyhow!(
            "vcf.call retained callers must stay complete across active-scope, command, output, parser, report, and smoke proof"
        ));
    }
    Ok(report)
}

fn build_vcf_call_ready_report(repo_root: &Path, output_path: &Path) -> Result<VcfCallReadyReport> {
    let retained_rows = collect_vcf_active_stage_tool_matrix_rows(repo_root)?
        .into_iter()
        .filter(|row| row.stage_id == VCF_CALL_STAGE_ID)
        .collect::<Vec<_>>();
    if retained_rows.is_empty() {
        return Err(anyhow!("VCF call readiness is missing retained `vcf.call` bindings"));
    }

    let active_rows = collect_all_domain_active_stage_tool_matrix_rows(repo_root)?
        .into_iter()
        .filter(|row| row.domain == "vcf" && row.stage_id == VCF_CALL_STAGE_ID)
        .collect::<Vec<_>>();
    let active_by_key = active_rows
        .iter()
        .cloned()
        .map(|row| (binding_key_from_active_row(&row), row))
        .collect::<BTreeMap<_, _>>();

    let command_report =
        render_vcf_commands(repo_root, PathBuf::from(DEFAULT_VCF_RENDERED_COMMANDS_PATH))?;
    let command_by_tool = command_report
        .rows
        .iter()
        .filter(|row| row.stage_id == VCF_CALL_STAGE_ID)
        .cloned()
        .map(|row| (row.tool_id.clone(), row))
        .collect::<BTreeMap<_, _>>();

    let output_by_tool = collect_vcf_adapter_output_coverage_rows(repo_root)?
        .into_iter()
        .filter(|row| row.stage_id == VCF_CALL_STAGE_ID)
        .map(|row| (row.tool_id.clone(), row))
        .collect::<BTreeMap<_, _>>();
    let parser_by_tool = collect_vcf_parser_coverage_rows(repo_root)?
        .2
        .into_iter()
        .filter(|row| row.stage_id == VCF_CALL_STAGE_ID)
        .map(|row| (row.tool_id.clone(), row))
        .collect::<BTreeMap<_, _>>();
    let expected_by_key = collect_vcf_expected_benchmark_result_rows(repo_root)?
        .into_iter()
        .filter(|row| row.stage_id == VCF_CALL_STAGE_ID)
        .map(|row| {
            (
                BindingKey {
                    stage_id: row.stage_id.clone(),
                    tool_id: row.tool_id.clone(),
                    corpus_id: row.corpus_id.clone(),
                    asset_profile_id: row.asset_profile_id.clone(),
                },
                row,
            )
        })
        .collect::<BTreeMap<_, _>>();
    let report_by_tool = collect_vcf_report_map_rows(repo_root)?
        .into_iter()
        .filter(|row| row.stage_id == VCF_CALL_STAGE_ID)
        .map(|row| (row.tool_id.clone(), row))
        .collect::<BTreeMap<_, _>>();

    let mut rows = Vec::with_capacity(retained_rows.len());
    for retained_row in retained_rows {
        let active_row = active_by_key.get(&binding_key_from_retained_row(&retained_row));
        let command_row = command_by_tool.get(&retained_row.tool_id);
        let output_row = output_by_tool.get(&retained_row.tool_id);
        let parser_row = parser_by_tool.get(&retained_row.tool_id);
        let expected_row = expected_by_key.get(&binding_key_from_retained_row(&retained_row));
        let report_row = report_by_tool.get(&retained_row.tool_id);
        let smoke_report = if retained_row.scope_state == "active" {
            run_local_vcf_call_smoke(repo_root, &retained_row.tool_id).ok()
        } else {
            None
        };
        rows.push(build_vcf_call_ready_row(
            &command_report,
            retained_row,
            active_row,
            command_row,
            output_row,
            parser_row,
            expected_row,
            report_row,
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

    let report = VcfCallReadyReport {
        schema_version: VCF_CALL_READY_SCHEMA_VERSION,
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
    ensure_vcf_call_ready_contract(&report)?;
    Ok(report)
}

#[allow(clippy::too_many_arguments)]
fn build_vcf_call_ready_row(
    command_report: &VcfRenderedCommandsReport,
    retained_row: VcfActiveStageToolMatrixRow,
    active_row: Option<&AllDomainActiveStageToolMatrixRow>,
    command_row: Option<&VcfRenderedCommandRow>,
    output_row: Option<&VcfAdapterOutputCoverageRow>,
    parser_row: Option<&VcfParserCoverageRow>,
    expected_row: Option<&VcfExpectedBenchmarkResultRow>,
    report_row: Option<&VcfReportMapRow>,
    smoke_report: Option<&LocalVcfCallSmokeReport>,
) -> VcfCallReadyRow {
    let result_id =
        expected_row.map(expected_result_id).unwrap_or_else(|| retained_result_id(&retained_row));
    let required_metric_names = required_metric_names();
    let mut missing_surfaces = Vec::new();

    let retained_scope_active = retained_row.scope_state == "active";
    if !retained_scope_active {
        missing_surfaces.push("retained_vcf_active_scope".to_string());
    }

    let all_domain_active_row_present = active_row.is_some();
    if !all_domain_active_row_present {
        missing_surfaces.push("all_domain_active_row".to_string());
    }

    let command_ready = command_row.is_some_and(|row| {
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

    let output_ready = output_row.is_some_and(|row| {
        row.status == VcfAdapterOutputCoverageStatus::Complete
            && row.benchmark_status == "benchmark_ready"
            && contains_artifact_id(&row.raw_outputs, "called_vcf")
            && !row.normalized_metrics.is_empty()
            && !row.manifest.trim().is_empty()
            && contains_artifact_id(&row.index_outputs, "called_vcf_tbi")
    });
    if !output_ready {
        missing_surfaces.push("vcf_adapter_output_coverage".to_string());
    }

    let parser_ready = parser_row.is_some_and(|row| {
        row.coverage_status == VcfParserCoverageStatus::Covered
            && !row.parser_id.trim().is_empty()
            && !row.fixture_path.trim().is_empty()
            && !row.schema_id.trim().is_empty()
    });
    if !parser_ready {
        missing_surfaces.push("vcf_parser_coverage".to_string());
    }

    let expected_result_ready = expected_row.is_some_and(|row| {
        row.report_section == "variant_calling"
            && row.expected_outputs.iter().any(|value| value == "called_vcf")
            && required_metric_names
                .iter()
                .all(|metric| row.expected_metrics.iter().any(|value| value == metric))
    });
    if !expected_result_ready {
        missing_surfaces.push("vcf_expected_benchmark_results".to_string());
    }

    let report_ready = report_row.is_some_and(|row| {
        row.section_id == "variant_calling"
            && row.summary_table == "variant_calling_metrics"
            && required_metric_names
                .iter()
                .all(|metric| row.metric_columns.iter().any(|value| value == metric))
    });
    if !report_ready {
        missing_surfaces.push("vcf_report_map".to_string());
    }

    let smoke_ready = smoke_report.is_some_and(|report| {
        report.parseable
            && report.gt_present
            && !report.output_vcf_path.trim().is_empty()
            && !report.output_tbi_path.trim().is_empty()
            && !report.metrics_path.trim().is_empty()
            && !report.stage_result_manifest_path.trim().is_empty()
            && report.sample_count > 0
            && report.variant_count >= report.snp_count
            && report.variant_count >= report.indel_count
    });
    if !smoke_ready {
        missing_surfaces.push("local_vcf_call_smoke".to_string());
    }

    let coverage_status = if missing_surfaces.is_empty() {
        COVERAGE_STATUS_COMPLETE.to_string()
    } else {
        COVERAGE_STATUS_INCOMPLETE.to_string()
    };
    let reason = if missing_surfaces.is_empty() {
        format!(
            "retained VCF caller `{}` keeps active scope, command, output, parser, expected-result, report, and smoke proof for `vcf.call`",
            retained_row.tool_id
        )
    } else {
        format!(
            "retained VCF caller `{}` is missing: {}",
            retained_row.tool_id,
            missing_surfaces.join(", ")
        )
    };

    VcfCallReadyRow {
        result_id,
        stage_id: retained_row.stage_id,
        tool_id: retained_row.tool_id,
        tool_status: retained_row.tool_status,
        corpus_id: retained_row.corpus_id,
        asset_profile_id: retained_row.asset_profile_id,
        adapter_id: retained_row.adapter_id,
        parser_id: retained_row.parser_id,
        schema_id: retained_row.schema_id,
        retained_scope_state: retained_row.scope_state,
        retained_scope_detail: retained_row.scope_detail,
        retained_scope_proof_path: retained_row.scope_proof_path,
        all_domain_active_row_present,
        all_domain_active_row_proof_path: DEFAULT_ALL_DOMAIN_ACTIVE_STAGE_TOOL_MATRIX_PATH
            .to_string(),
        command_ready,
        command_source: command_row
            .map(|row| row.command_source.clone())
            .unwrap_or_else(no_value_string),
        command_step_count: command_row.map(|row| row.command_steps.len()).unwrap_or(0),
        command_step_ids: command_row
            .map(|row| row.command_steps.iter().map(|step| step.step_id.clone()).collect())
            .unwrap_or_default(),
        primary_executables: command_row
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
        raw_outputs: output_row.map(|row| row.raw_outputs.clone()).unwrap_or_default(),
        normalized_metrics_outputs: output_row
            .map(|row| row.normalized_metrics.clone())
            .unwrap_or_default(),
        manifest_output: output_row.map(|row| row.manifest.clone()).unwrap_or_else(no_value_string),
        index_outputs: output_row.map(|row| row.index_outputs.clone()).unwrap_or_default(),
        parser_ready,
        parser_proof_path: DEFAULT_VCF_PARSER_COVERAGE_PATH.to_string(),
        parser_fixture_parser_id: parser_row
            .map(|row| row.parser_id.clone())
            .unwrap_or_else(no_value_string),
        parser_fixture_schema_id: parser_row
            .map(|row| row.schema_id.clone())
            .unwrap_or_else(no_value_string),
        parser_fixture_path: parser_row
            .map(|row| row.fixture_path.clone())
            .unwrap_or_else(no_value_string),
        expected_result_ready,
        expected_result_proof_path: DEFAULT_VCF_EXPECTED_BENCHMARK_RESULTS_PATH.to_string(),
        expected_outputs: expected_row.map(|row| row.expected_outputs.clone()).unwrap_or_default(),
        expected_metrics: expected_row.map(|row| row.expected_metrics.clone()).unwrap_or_default(),
        report_section_id: expected_row
            .map(|row| row.report_section.clone())
            .unwrap_or_else(no_value_string),
        report_ready,
        report_map_proof_path: DEFAULT_VCF_REPORT_MAP_PATH.to_string(),
        summary_table_id: report_row
            .map(|row| row.summary_table.clone())
            .unwrap_or_else(no_value_string),
        report_metric_columns: report_row.map(|row| row.metric_columns.clone()).unwrap_or_default(),
        smoke_ready,
        smoke_command: smoke_report
            .map(|report| report.command.clone())
            .unwrap_or_else(no_value_string),
        smoke_output_root: smoke_report
            .map(|report| report.output_root.clone())
            .unwrap_or_else(no_value_string),
        smoke_output_vcf_path: smoke_report
            .map(|report| report.output_vcf_path.clone())
            .unwrap_or_else(no_value_string),
        smoke_output_tbi_path: smoke_report
            .map(|report| report.output_tbi_path.clone())
            .unwrap_or_else(no_value_string),
        smoke_metrics_path: smoke_report
            .map(|report| report.metrics_path.clone())
            .unwrap_or_else(no_value_string),
        smoke_stage_result_manifest_path: smoke_report
            .map(|report| report.stage_result_manifest_path.clone())
            .unwrap_or_else(no_value_string),
        smoke_parseable: smoke_report.is_some_and(|report| report.parseable),
        smoke_gt_present: smoke_report.is_some_and(|report| report.gt_present),
        smoke_gl_present: smoke_report.is_some_and(|report| report.gl_present),
        smoke_variant_count: smoke_report.map(|report| report.variant_count).unwrap_or(0),
        smoke_snp_count: smoke_report.map(|report| report.snp_count).unwrap_or(0),
        smoke_indel_count: smoke_report.map(|report| report.indel_count).unwrap_or(0),
        smoke_sample_count: smoke_report.map(|report| report.sample_count).unwrap_or(0),
        required_metric_names,
        missing_surfaces,
        coverage_status,
        reason,
    }
}

fn ensure_vcf_call_ready_contract(report: &VcfCallReadyReport) -> Result<()> {
    if report.retained_row_count != report.rows.len() {
        return Err(anyhow!(
            "VCF call readiness must keep exactly one row per retained `vcf.call` binding"
        ));
    }
    if report.rows.is_empty() {
        return Err(anyhow!("VCF call readiness must keep at least one retained caller row"));
    }
    if report.checked_surface_count != 8 {
        return Err(anyhow!("VCF call readiness must record exactly 8 checked surfaces"));
    }
    let unique_results =
        report.rows.iter().map(|row| row.result_id.as_str()).collect::<BTreeSet<_>>().len();
    if unique_results != report.rows.len() {
        return Err(anyhow!(
            "VCF call readiness must keep one unique result_id per retained caller row"
        ));
    }
    for row in &report.rows {
        if row.stage_id != VCF_CALL_STAGE_ID {
            return Err(anyhow!(
                "VCF call readiness row `{}` drifted away from the `vcf.call` stage",
                row.stage_id
            ));
        }
        if row.coverage_status == COVERAGE_STATUS_COMPLETE && !row.missing_surfaces.is_empty() {
            return Err(anyhow!(
                "VCF call readiness row `{}` / `{}` cannot be complete while listing missing surfaces",
                row.stage_id,
                row.tool_id
            ));
        }
    }
    Ok(())
}

fn binding_key_from_retained_row(row: &VcfActiveStageToolMatrixRow) -> BindingKey {
    BindingKey {
        stage_id: row.stage_id.clone(),
        tool_id: row.tool_id.clone(),
        corpus_id: row.corpus_id.clone(),
        asset_profile_id: row.asset_profile_id.clone(),
    }
}

fn binding_key_from_active_row(row: &AllDomainActiveStageToolMatrixRow) -> BindingKey {
    BindingKey {
        stage_id: row.stage_id.clone(),
        tool_id: row.tool_id.clone(),
        corpus_id: row.corpus_id.clone(),
        asset_profile_id: row.asset_profile_id.clone(),
    }
}

fn expected_result_id(row: &VcfExpectedBenchmarkResultRow) -> String {
    format!("vcf:{}:{}:{}:{}", row.corpus_id, row.stage_id, row.asset_profile_id, row.tool_id)
}

fn retained_result_id(row: &VcfActiveStageToolMatrixRow) -> String {
    format!("vcf:{}:{}:{}:{}", row.corpus_id, row.stage_id, row.asset_profile_id, row.tool_id)
}

fn contains_artifact_id(entries: &[String], expected_id: &str) -> bool {
    entries.iter().any(|entry| artifact_id(entry) == expected_id)
}

fn artifact_id(entry: &str) -> &str {
    entry.split_once('=').map(|(id, _)| id).unwrap_or(entry)
}

fn required_metric_names() -> Vec<String> {
    REQUIRED_METRIC_NAMES.iter().map(|value| (*value).to_string()).collect()
}

fn no_value_string() -> String {
    NO_VALUE.to_string()
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
