use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use super::all_domain_active_stage_tool_matrix::DEFAULT_ALL_DOMAIN_ACTIVE_STAGE_TOOL_MATRIX_PATH;
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
use crate::commands::benchmark::local_vcf_prepare_reference_panel_smoke::{
    run_local_vcf_prepare_reference_panel_smoke, LocalVcfPrepareReferencePanelSmokeReport,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_VCF_PREPARE_REFERENCE_PANEL_READY_PATH: &str =
    "benchmarks/readiness/vcf/prepare-reference-panel-ready.json";
const VCF_PREPARE_REFERENCE_PANEL_READY_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.vcf_prepare_reference_panel_ready.v1";
const VCF_PREPARE_REFERENCE_PANEL_STAGE_ID: &str = "vcf.prepare_reference_panel";
const REQUIRED_METRIC_NAMES: [&str; 5] = [
    "input_variants",
    "output_variants",
    "sample_count",
    "duplicate_sites_removed",
    "normalization_status",
];
const REQUIRED_OUTPUT_VARIANT_COUNT_MINIMUM: u64 = 1;
const REQUIRED_NORMALIZATION_STATUS: &str = "sorted_indexed_deduplicated";
const COVERAGE_STATUS_COMPLETE: &str = "complete";
const COVERAGE_STATUS_INCOMPLETE: &str = "incomplete";
const NO_VALUE: &str = "none";

#[derive(Debug, Clone, Serialize)]
pub(crate) struct VcfPrepareReferencePanelReadyRow {
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
    pub(crate) smoke_input_vcf_path: String,
    pub(crate) smoke_raw_panel_path: String,
    pub(crate) smoke_panel_vcf_path: String,
    pub(crate) smoke_panel_tbi_path: String,
    pub(crate) smoke_panel_manifest_path: String,
    pub(crate) smoke_overlap_path: String,
    pub(crate) smoke_panel_overlap_path: String,
    pub(crate) smoke_panel_files_path: String,
    pub(crate) smoke_overlap_tsv_path: String,
    pub(crate) smoke_chunks_path: String,
    pub(crate) smoke_metrics_path: String,
    pub(crate) smoke_stage_result_manifest_path: String,
    pub(crate) smoke_panel_id: String,
    pub(crate) smoke_map_id: String,
    pub(crate) smoke_input_variants: u64,
    pub(crate) smoke_output_variants: u64,
    pub(crate) smoke_sample_count: u64,
    pub(crate) smoke_sample_ids: Vec<String>,
    pub(crate) smoke_sample_consistent: bool,
    pub(crate) smoke_duplicate_sites_removed: u64,
    pub(crate) smoke_normalization_status: String,
    pub(crate) smoke_index_path: String,
    pub(crate) smoke_parseable: bool,
    pub(crate) smoke_gt_present: bool,
    pub(crate) smoke_gl_present: bool,
    pub(crate) smoke_validation_checks: BTreeMap<String, bool>,
    pub(crate) required_metric_names: Vec<String>,
    pub(crate) missing_surfaces: Vec<String>,
    pub(crate) coverage_status: String,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct VcfPrepareReferencePanelReadyReport {
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
    pub(crate) rows: Vec<VcfPrepareReferencePanelReadyRow>,
    pub(crate) violations: Vec<VcfPrepareReferencePanelReadyRow>,
}

pub(crate) fn run_render_vcf_prepare_reference_panel_ready(
    args: &parse::BenchReadinessRenderVcfPrepareReferencePanelReadyArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_vcf_prepare_reference_panel_ready(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_PREPARE_REFERENCE_PANEL_READY_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_vcf_prepare_reference_panel_ready(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<VcfPrepareReferencePanelReadyReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let report = build_vcf_prepare_reference_panel_ready_report(repo_root, &output_path)?;
    bijux_dna_infra::atomic_write_json(&output_path, &report)?;
    if !report.ok {
        return Err(anyhow!(
            "vcf.prepare_reference_panel active retained callers must stay complete across active-scope, command, output, parser, report, and panel-preparation smoke proof"
        ));
    }
    Ok(report)
}

fn build_vcf_prepare_reference_panel_ready_report(
    repo_root: &Path,
    output_path: &Path,
) -> Result<VcfPrepareReferencePanelReadyReport> {
    let (command_report, bindings) =
        collect_vcf_stage_readiness_bindings(repo_root, VCF_PREPARE_REFERENCE_PANEL_STAGE_ID)?;
    let active_bindings = bindings
        .into_iter()
        .filter(|binding| binding.retained_row.scope_state == "active")
        .collect::<Vec<_>>();

    let mut rows = Vec::with_capacity(active_bindings.len());
    for binding in active_bindings {
        let smoke_report =
            run_local_vcf_prepare_reference_panel_smoke(repo_root, &binding.retained_row.tool_id)
                .ok();
        rows.push(build_vcf_prepare_reference_panel_ready_row(
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

    let report = VcfPrepareReferencePanelReadyReport {
        schema_version: VCF_PREPARE_REFERENCE_PANEL_READY_SCHEMA_VERSION,
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
    ensure_vcf_prepare_reference_panel_ready_contract(&report)?;
    Ok(report)
}

fn build_vcf_prepare_reference_panel_ready_row(
    command_report: &VcfRenderedCommandsReport,
    binding: VcfStageReadinessBinding,
    smoke_report: Option<&LocalVcfPrepareReferencePanelSmokeReport>,
) -> VcfPrepareReferencePanelReadyRow {
    let result_id = binding
        .expected_row
        .as_ref()
        .map(expected_result_id)
        .unwrap_or_else(|| retained_result_id(&binding));
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
            && contains_artifact_id(&row.raw_outputs, "prepared_panel")
            && contains_artifact_id(&row.normalized_metrics, "chunks_json")
            && !row.manifest.trim().is_empty()
            && contains_artifact_id(&row.index_outputs, "prepared_panel_tbi")
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
        row.report_section == "reference_panel_preparation"
            && row.expected_outputs.iter().any(|value| value == "prepared_panel")
            && required_metric_names
                .iter()
                .all(|metric| row.expected_metrics.iter().any(|value| value == metric))
    });
    if !expected_result_ready {
        missing_surfaces.push("vcf_expected_benchmark_results".to_string());
    }

    let report_ready = binding.report_row.as_ref().is_some_and(|row| {
        row.section_id == "reference_panel_preparation"
            && row.summary_table == "reference_panel_readiness"
            && required_metric_names
                .iter()
                .all(|metric| row.metric_columns.iter().any(|value| value == metric))
    });
    if !report_ready {
        missing_surfaces.push("vcf_report_map".to_string());
    }

    let smoke_ready = smoke_report.is_some_and(|report| {
        report.parseable
            && report.validation_checks.get("bgzip") == Some(&true)
            && report.validation_checks.get("tabix_index") == Some(&true)
            && report.validation_checks.get("sorted") == Some(&true)
            && report.output_variants >= REQUIRED_OUTPUT_VARIANT_COUNT_MINIMUM
            && report.input_variants > report.output_variants
            && report.duplicate_sites_removed == report.input_variants - report.output_variants
            && report.sample_count >= 1
            && report.sample_consistent
            && report.normalization_status == REQUIRED_NORMALIZATION_STATUS
            && !report.panel_vcf_path.trim().is_empty()
            && !report.panel_tbi_path.trim().is_empty()
    });
    if !smoke_ready {
        missing_surfaces.push("local_vcf_prepare_reference_panel_smoke".to_string());
    }

    let coverage_status = if missing_surfaces.is_empty() {
        COVERAGE_STATUS_COMPLETE.to_string()
    } else {
        COVERAGE_STATUS_INCOMPLETE.to_string()
    };
    let reason = if coverage_status == COVERAGE_STATUS_COMPLETE {
        format!(
            "retained VCF caller `{}` keeps active scope, command, output, parser, expected-result, report, and smoke proof for `{}`",
            binding.retained_row.tool_id, binding.retained_row.stage_id
        )
    } else {
        format!(
            "retained VCF caller `{}` is missing {} for `{}`",
            binding.retained_row.tool_id,
            missing_surfaces.join(", "),
            binding.retained_row.stage_id
        )
    };

    VcfPrepareReferencePanelReadyRow {
        result_id,
        stage_id: binding.retained_row.stage_id.clone(),
        tool_id: binding.retained_row.tool_id.clone(),
        tool_status: binding.retained_row.tool_status.clone(),
        corpus_id: binding.retained_row.corpus_id.clone(),
        asset_profile_id: binding.retained_row.asset_profile_id.clone(),
        adapter_id: binding.retained_row.adapter_id.clone(),
        parser_id: binding.retained_row.parser_id.clone(),
        schema_id: binding.retained_row.schema_id.clone(),
        retained_scope_state: binding.retained_row.scope_state.clone(),
        retained_scope_detail: binding.retained_row.scope_detail.clone(),
        retained_scope_proof_path: binding.retained_row.scope_proof_path.clone(),
        all_domain_active_row_present,
        all_domain_active_row_proof_path: if all_domain_active_row_present {
            DEFAULT_ALL_DOMAIN_ACTIVE_STAGE_TOOL_MATRIX_PATH.to_string()
        } else {
            NO_VALUE.to_string()
        },
        command_ready,
        command_source: "vcf_bcftools_adapter".to_string(),
        command_step_count: binding
            .command_row
            .as_ref()
            .map(|row| row.command_steps.len())
            .unwrap_or(0),
        command_step_ids: binding
            .command_row
            .as_ref()
            .map(|row| row.command_steps.iter().map(|step| step.step_id.clone()).collect())
            .unwrap_or_default(),
        primary_executables: binding
            .command_row
            .as_ref()
            .map(primary_executables_from_binding)
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
            .map(|row| row.manifest.clone())
            .unwrap_or_else(|| NO_VALUE.to_string()),
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
            .map(|row| row.parser_fixture_parser_id.clone())
            .unwrap_or_else(|| NO_VALUE.to_string()),
        parser_fixture_schema_id: binding
            .parser_row
            .as_ref()
            .map(|row| row.parser_fixture_schema_id.clone())
            .unwrap_or_else(|| NO_VALUE.to_string()),
        parser_fixture_path: binding
            .parser_row
            .as_ref()
            .map(|row| row.parser_fixture_root_path.clone())
            .unwrap_or_else(|| NO_VALUE.to_string()),
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
            .report_row
            .as_ref()
            .map(|row| row.section_id.clone())
            .unwrap_or_else(|| NO_VALUE.to_string()),
        report_ready,
        report_map_proof_path: DEFAULT_VCF_REPORT_MAP_PATH.to_string(),
        summary_table_id: binding
            .report_row
            .as_ref()
            .map(|row| row.summary_table.clone())
            .unwrap_or_else(|| NO_VALUE.to_string()),
        report_metric_columns: binding
            .report_row
            .as_ref()
            .map(|row| row.metric_columns.clone())
            .unwrap_or_default(),
        smoke_ready,
        smoke_command: smoke_report
            .map(|report| report.command.clone())
            .unwrap_or_else(|| NO_VALUE.to_string()),
        smoke_output_root: smoke_report
            .map(|report| report.output_root.clone())
            .unwrap_or_else(|| NO_VALUE.to_string()),
        smoke_input_vcf_path: smoke_report
            .map(|report| report.input_vcf_path.clone())
            .unwrap_or_else(|| NO_VALUE.to_string()),
        smoke_raw_panel_path: smoke_report
            .map(|report| report.raw_panel_path.clone())
            .unwrap_or_else(|| NO_VALUE.to_string()),
        smoke_panel_vcf_path: smoke_report
            .map(|report| report.panel_vcf_path.clone())
            .unwrap_or_else(|| NO_VALUE.to_string()),
        smoke_panel_tbi_path: smoke_report
            .map(|report| report.panel_tbi_path.clone())
            .unwrap_or_else(|| NO_VALUE.to_string()),
        smoke_panel_manifest_path: smoke_report
            .map(|report| report.panel_manifest_path.clone())
            .unwrap_or_else(|| NO_VALUE.to_string()),
        smoke_overlap_path: smoke_report
            .map(|report| report.overlap_path.clone())
            .unwrap_or_else(|| NO_VALUE.to_string()),
        smoke_panel_overlap_path: smoke_report
            .map(|report| report.panel_overlap_path.clone())
            .unwrap_or_else(|| NO_VALUE.to_string()),
        smoke_panel_files_path: smoke_report
            .map(|report| report.panel_files_path.clone())
            .unwrap_or_else(|| NO_VALUE.to_string()),
        smoke_overlap_tsv_path: smoke_report
            .map(|report| report.overlap_tsv_path.clone())
            .unwrap_or_else(|| NO_VALUE.to_string()),
        smoke_chunks_path: smoke_report
            .map(|report| report.chunks_path.clone())
            .unwrap_or_else(|| NO_VALUE.to_string()),
        smoke_metrics_path: smoke_report
            .map(|report| report.metrics_path.clone())
            .unwrap_or_else(|| NO_VALUE.to_string()),
        smoke_stage_result_manifest_path: smoke_report
            .map(|report| report.stage_result_manifest_path.clone())
            .unwrap_or_else(|| NO_VALUE.to_string()),
        smoke_panel_id: smoke_report
            .map(|report| report.panel_id.clone())
            .unwrap_or_else(|| NO_VALUE.to_string()),
        smoke_map_id: smoke_report
            .map(|report| report.map_id.clone())
            .unwrap_or_else(|| NO_VALUE.to_string()),
        smoke_input_variants: smoke_report.map(|report| report.input_variants).unwrap_or(0),
        smoke_output_variants: smoke_report.map(|report| report.output_variants).unwrap_or(0),
        smoke_sample_count: smoke_report.map(|report| report.sample_count).unwrap_or(0),
        smoke_sample_ids: smoke_report.map(|report| report.sample_ids.clone()).unwrap_or_default(),
        smoke_sample_consistent: smoke_report
            .map(|report| report.sample_consistent)
            .unwrap_or(false),
        smoke_duplicate_sites_removed: smoke_report
            .map(|report| report.duplicate_sites_removed)
            .unwrap_or(0),
        smoke_normalization_status: smoke_report
            .map(|report| report.normalization_status.clone())
            .unwrap_or_else(|| NO_VALUE.to_string()),
        smoke_index_path: smoke_report
            .map(|report| report.index_path.clone())
            .unwrap_or_else(|| NO_VALUE.to_string()),
        smoke_parseable: smoke_report.map(|report| report.parseable).unwrap_or(false),
        smoke_gt_present: smoke_report.map(|report| report.gt_present).unwrap_or(false),
        smoke_gl_present: smoke_report.map(|report| report.gl_present).unwrap_or(false),
        smoke_validation_checks: smoke_report
            .map(|report| report.validation_checks.clone())
            .unwrap_or_default(),
        required_metric_names,
        missing_surfaces,
        coverage_status,
        reason,
    }
}

fn ensure_vcf_prepare_reference_panel_ready_contract(
    report: &VcfPrepareReferencePanelReadyReport,
) -> Result<()> {
    if report.retained_row_count != 1
        || report.active_row_count != 1
        || report.complete_row_count != 1
        || report.incomplete_row_count != 0
        || report.checked_surface_count != 8
        || report.violation_count != 0
        || !report.ok
    {
        return Err(anyhow!(
            "vcf.prepare_reference_panel readiness counts drifted from the governed single-row active completion contract"
        ));
    }

    let row = report.rows.first().ok_or_else(|| {
        anyhow!("vcf.prepare_reference_panel readiness is missing its governed row")
    })?;
    if row.stage_id != VCF_PREPARE_REFERENCE_PANEL_STAGE_ID
        || row.tool_id != "bcftools"
        || row.report_section_id != "reference_panel_preparation"
        || row.summary_table_id != "reference_panel_readiness"
        || row.coverage_status != COVERAGE_STATUS_COMPLETE
        || row.smoke_normalization_status != REQUIRED_NORMALIZATION_STATUS
        || row.smoke_output_variants < REQUIRED_OUTPUT_VARIANT_COUNT_MINIMUM
        || !row.smoke_sample_consistent
        || !row.smoke_parseable
        || row.smoke_validation_checks.get("bgzip") != Some(&true)
        || row.smoke_validation_checks.get("tabix_index") != Some(&true)
        || row.smoke_validation_checks.get("sorted") != Some(&true)
    {
        return Err(anyhow!(
            "vcf.prepare_reference_panel readiness row drifted from the governed panel-preparation contract"
        ));
    }
    Ok(())
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

fn expected_result_id(
    binding: &super::vcf_expected_benchmark_results::VcfExpectedBenchmarkResultRow,
) -> String {
    format!(
        "vcf:{}:{}:{}:{}",
        binding.corpus_id, binding.stage_id, binding.asset_profile_id, binding.tool_id
    )
}

fn primary_executables_from_binding(
    row: &super::vcf_rendered_command_rows::VcfRenderedCommandRow,
) -> Vec<String> {
    let mut values = BTreeSet::new();
    for step in &row.command_steps {
        if let Some(executable) = step.argv.first() {
            values.insert(executable.clone());
        }
    }
    values.into_iter().collect()
}

fn contains_artifact_id(values: &[String], artifact_id: &str) -> bool {
    values.iter().any(|value| value.split('=').next() == Some(artifact_id))
}

fn required_metric_names() -> Vec<String> {
    REQUIRED_METRIC_NAMES.iter().map(|metric| (*metric).to_string()).collect()
}

fn repo_relative_path(repo_root: &Path, candidate: &Path) -> PathBuf {
    if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        repo_root.join(candidate)
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}
