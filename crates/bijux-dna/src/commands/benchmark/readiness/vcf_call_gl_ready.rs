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
use crate::commands::benchmark::local_vcf_call_gl_smoke::{
    run_local_vcf_call_gl_smoke, LocalVcfCallGlSmokeReport,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_VCF_CALL_GL_READY_PATH: &str =
    "benchmarks/readiness/vcf/call-gl-ready.json";
const VCF_CALL_GL_READY_SCHEMA_VERSION: &str = "bijux.bench.readiness.vcf_call_gl_ready.v1";
const VCF_CALL_GL_STAGE_ID: &str = "vcf.call_gl";
const REQUIRED_METRIC_NAMES: [&str; 5] = [
    "likelihood_field",
    "sites_with_likelihoods",
    "samples_with_likelihoods",
    "missing_likelihoods",
    "sample_count",
];
const COVERAGE_STATUS_COMPLETE: &str = "complete";
const COVERAGE_STATUS_INCOMPLETE: &str = "incomplete";
const NO_VALUE: &str = "none";

#[derive(Debug, Clone, Serialize)]
pub(crate) struct VcfCallGlReadyRow {
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
    pub(crate) smoke_likelihood_field: String,
    pub(crate) smoke_sites_with_likelihoods: u64,
    pub(crate) smoke_samples_with_likelihoods: u64,
    pub(crate) smoke_missing_likelihoods: u64,
    pub(crate) smoke_sample_count: u64,
    pub(crate) required_metric_names: Vec<String>,
    pub(crate) missing_surfaces: Vec<String>,
    pub(crate) coverage_status: String,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct VcfCallGlReadyReport {
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
    pub(crate) rows: Vec<VcfCallGlReadyRow>,
    pub(crate) violations: Vec<VcfCallGlReadyRow>,
}

pub(crate) fn run_render_vcf_call_gl_ready(
    args: &parse::BenchReadinessRenderVcfCallGlReadyArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_vcf_call_gl_ready(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_CALL_GL_READY_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_vcf_call_gl_ready(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<VcfCallGlReadyReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let report = build_vcf_call_gl_ready_report(repo_root, &output_path)?;
    bijux_dna_infra::atomic_write_json(&output_path, &report)?;
    if !report.ok {
        return Err(anyhow!(
            "vcf.call_gl active retained callers must stay complete across active-scope, command, output, parser, report, and genotype-likelihood smoke proof"
        ));
    }
    Ok(report)
}

fn build_vcf_call_gl_ready_report(
    repo_root: &Path,
    output_path: &Path,
) -> Result<VcfCallGlReadyReport> {
    let (command_report, bindings) =
        collect_vcf_stage_readiness_bindings(repo_root, VCF_CALL_GL_STAGE_ID)?;
    let active_bindings = bindings
        .into_iter()
        .filter(|binding| binding.retained_row.scope_state == "active")
        .collect::<Vec<_>>();

    let mut rows = Vec::with_capacity(active_bindings.len());
    for binding in active_bindings {
        let smoke_report =
            run_local_vcf_call_gl_smoke(repo_root, &binding.retained_row.tool_id).ok();
        rows.push(build_vcf_call_gl_ready_row(&command_report, binding, smoke_report.as_ref()));
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

    let report = VcfCallGlReadyReport {
        schema_version: VCF_CALL_GL_READY_SCHEMA_VERSION,
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
    ensure_vcf_call_gl_ready_contract(&report)?;
    Ok(report)
}

fn build_vcf_call_gl_ready_row(
    command_report: &VcfRenderedCommandsReport,
    binding: VcfStageReadinessBinding,
    smoke_report: Option<&LocalVcfCallGlSmokeReport>,
) -> VcfCallGlReadyRow {
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
            && contains_artifact_id(&row.raw_outputs, "gl_sites_vcf")
            && !row.normalized_metrics.is_empty()
            && !row.manifest.trim().is_empty()
            && contains_artifact_id(&row.index_outputs, "gl_sites_vcf_tbi")
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
        row.report_section == "variant_calling"
            && row.expected_outputs.iter().any(|value| value == "gl_sites_vcf")
            && required_metric_names
                .iter()
                .all(|metric| row.expected_metrics.iter().any(|value| value == metric))
    });
    if !expected_result_ready {
        missing_surfaces.push("vcf_expected_benchmark_results".to_string());
    }

    let report_ready = binding.report_row.as_ref().is_some_and(|row| {
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
            && !report.gt_present
            && report.gl_present
            && !report.output_vcf_path.trim().is_empty()
            && !report.output_tbi_path.trim().is_empty()
            && !report.metrics_path.trim().is_empty()
            && !report.stage_result_manifest_path.trim().is_empty()
            && matches!(report.likelihood_field.as_str(), "GL" | "GP" | "PL")
            && report.sites_with_likelihoods > 0
            && report.samples_with_likelihoods > 0
            && report.sample_count > 0
    });
    if !smoke_ready {
        missing_surfaces.push("local_vcf_call_gl_smoke".to_string());
    }

    let coverage_status = if missing_surfaces.is_empty() {
        COVERAGE_STATUS_COMPLETE.to_string()
    } else {
        COVERAGE_STATUS_INCOMPLETE.to_string()
    };
    let reason = if missing_surfaces.is_empty() {
        format!(
            "active retained VCF GL caller `{}` keeps active scope, command, output, parser, expected-result, report, and likelihood-bearing smoke proof for `vcf.call_gl`",
            binding.retained_row.tool_id
        )
    } else {
        format!(
            "active retained VCF GL caller `{}` is missing: {}",
            binding.retained_row.tool_id,
            missing_surfaces.join(", ")
        )
    };

    VcfCallGlReadyRow {
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
            .map(|row| row.command_source.clone())
            .unwrap_or_else(no_value_string),
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
            .map(|row| row.manifest.clone())
            .unwrap_or_else(no_value_string),
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
            .unwrap_or_else(no_value_string),
        parser_fixture_schema_id: binding
            .parser_row
            .as_ref()
            .map(|row| row.parser_fixture_schema_id.clone())
            .unwrap_or_else(no_value_string),
        parser_fixture_path: binding
            .parser_row
            .as_ref()
            .map(|row| row.parser_fixture_root_path.clone())
            .unwrap_or_else(no_value_string),
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
            .map(|row| row.report_section.clone())
            .unwrap_or_else(no_value_string),
        report_ready,
        report_map_proof_path: DEFAULT_VCF_REPORT_MAP_PATH.to_string(),
        summary_table_id: binding
            .report_row
            .as_ref()
            .map(|row| row.summary_table.clone())
            .unwrap_or_else(no_value_string),
        report_metric_columns: binding
            .report_row
            .as_ref()
            .map(|row| row.metric_columns.clone())
            .unwrap_or_default(),
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
        smoke_likelihood_field: smoke_report
            .map(|report| report.likelihood_field.clone())
            .unwrap_or_else(no_value_string),
        smoke_sites_with_likelihoods: smoke_report
            .map(|report| report.sites_with_likelihoods)
            .unwrap_or(0),
        smoke_samples_with_likelihoods: smoke_report
            .map(|report| report.samples_with_likelihoods)
            .unwrap_or(0),
        smoke_missing_likelihoods: smoke_report
            .map(|report| report.missing_likelihoods)
            .unwrap_or(0),
        smoke_sample_count: smoke_report.map(|report| report.sample_count).unwrap_or(0),
        required_metric_names,
        missing_surfaces,
        coverage_status,
        reason,
    }
}

fn ensure_vcf_call_gl_ready_contract(report: &VcfCallGlReadyReport) -> Result<()> {
    if report.retained_row_count != report.rows.len() {
        return Err(anyhow!(
            "VCF call_gl readiness must keep exactly one row per active retained `vcf.call_gl` binding"
        ));
    }
    if report.rows.is_empty() {
        return Err(anyhow!(
            "VCF call_gl readiness must keep at least one active retained caller row"
        ));
    }
    if report.checked_surface_count != 8 {
        return Err(anyhow!("VCF call_gl readiness must record exactly 8 checked surfaces"));
    }
    let unique_results =
        report.rows.iter().map(|row| row.result_id.as_str()).collect::<BTreeSet<_>>().len();
    if unique_results != report.rows.len() {
        return Err(anyhow!(
            "VCF call_gl readiness must keep one unique result_id per active retained caller row"
        ));
    }
    for row in &report.rows {
        if row.stage_id != VCF_CALL_GL_STAGE_ID {
            return Err(anyhow!(
                "VCF call_gl readiness row `{}` drifted away from the `vcf.call_gl` stage",
                row.stage_id
            ));
        }
        if row.coverage_status == COVERAGE_STATUS_COMPLETE && !row.missing_surfaces.is_empty() {
            return Err(anyhow!(
                "VCF call_gl readiness row `{}` / `{}` cannot be complete while listing missing surfaces",
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
