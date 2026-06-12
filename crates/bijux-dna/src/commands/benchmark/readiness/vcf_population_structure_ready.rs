use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::time::Duration;

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
use crate::commands::benchmark::local_vcf_population_structure_smoke::{
    run_local_vcf_population_structure_smoke, ConsumedAdmixtureReport, ConsumedPcaReport,
    LocalVcfPopulationStructureDistanceSummary, LocalVcfPopulationStructureSampleGroup,
    LocalVcfPopulationStructureSmokeReport,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_VCF_POPULATION_STRUCTURE_READY_PATH: &str =
    "benchmarks/readiness/vcf/population-structure-ready.json";
const VCF_POPULATION_STRUCTURE_READY_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.vcf_population_structure_ready.v1";
const VCF_POPULATION_STRUCTURE_STAGE_ID: &str = "vcf.population_structure";
const REQUIRED_METRIC_NAMES: [&str; 4] =
    ["sample_count", "pair_count", "within_population_pair_count", "cross_population_pair_count"];
const REQUIRED_STATUS: &str = "complete";
const REQUIRED_SAMPLE_COUNT: u64 = 4;
const REQUIRED_PAIR_COUNT: u64 = 6;
const REQUIRED_WITHIN_POPULATION_PAIR_COUNT: u64 = 2;
const REQUIRED_CROSS_POPULATION_PAIR_COUNT: u64 = 4;
const REQUIRED_SELECTED_K: u64 = 2;
const REQUIRED_SAMPLE_IDS: [&str; 4] = ["sample_a", "sample_b", "sample_c", "sample_d"];
const COVERAGE_STATUS_COMPLETE: &str = "complete";
const COVERAGE_STATUS_INCOMPLETE: &str = "incomplete";
const NO_VALUE: &str = "none";
const VCF_POPULATION_STRUCTURE_READY_LOCK_PATH: &str =
    "artifacts/bench-readiness/vcf-population-structure-ready/render.lock";

#[derive(Debug, Clone, Serialize)]
pub(crate) struct VcfPopulationStructureReadyRow {
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
    pub(crate) smoke_population_structure_json_path: String,
    pub(crate) smoke_source_population_structure_path: String,
    pub(crate) smoke_source_pruned_variants_path: String,
    pub(crate) smoke_source_logs_path: String,
    pub(crate) smoke_source_pca_report_path: String,
    pub(crate) smoke_source_admixture_report_path: String,
    pub(crate) smoke_stage_result_manifest_path: String,
    pub(crate) smoke_status: String,
    pub(crate) smoke_consumed_pca: Option<ConsumedPcaReport>,
    pub(crate) smoke_consumed_admixture: Option<ConsumedAdmixtureReport>,
    pub(crate) smoke_sample_group_count: usize,
    pub(crate) smoke_sample_groups: Vec<LocalVcfPopulationStructureSampleGroup>,
    pub(crate) smoke_distance_summary: Option<LocalVcfPopulationStructureDistanceSummary>,
    pub(crate) required_metric_names: Vec<String>,
    pub(crate) missing_surfaces: Vec<String>,
    pub(crate) coverage_status: String,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct VcfPopulationStructureReadyReport {
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
    pub(crate) rows: Vec<VcfPopulationStructureReadyRow>,
    pub(crate) violations: Vec<VcfPopulationStructureReadyRow>,
}

pub(crate) fn run_render_vcf_population_structure_ready(
    args: &parse::BenchReadinessRenderVcfPopulationStructureReadyArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_vcf_population_structure_ready(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_POPULATION_STRUCTURE_READY_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_vcf_population_structure_ready(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<VcfPopulationStructureReadyReport> {
    let _lock = bijux_dna_infra::FileLock::acquire(
        &repo_root.join(VCF_POPULATION_STRUCTURE_READY_LOCK_PATH),
        Duration::from_secs(300),
    )
    .with_context(|| {
        format!(
            "acquire VCF population-structure readiness lock {}",
            repo_root.join(VCF_POPULATION_STRUCTURE_READY_LOCK_PATH).display()
        )
    })?;
    let output_path = repo_relative_path(repo_root, &output_path);
    let report = build_vcf_population_structure_ready_report(repo_root, &output_path)?;
    bijux_dna_infra::atomic_write_json(&output_path, &report)?;
    if !report.ok {
        return Err(anyhow!(
            "vcf.population_structure active retained callers must stay complete across active-scope, command, output, parser, report, and consumed PCA/admixture smoke proof"
        ));
    }
    Ok(report)
}

fn build_vcf_population_structure_ready_report(
    repo_root: &Path,
    output_path: &Path,
) -> Result<VcfPopulationStructureReadyReport> {
    let (command_report, bindings) =
        collect_vcf_stage_readiness_bindings(repo_root, VCF_POPULATION_STRUCTURE_STAGE_ID)?;
    let active_bindings = bindings
        .into_iter()
        .filter(|binding| binding.retained_row.scope_state == "active")
        .collect::<Vec<_>>();

    let mut rows = Vec::with_capacity(active_bindings.len());
    for binding in active_bindings {
        let smoke_report =
            run_local_vcf_population_structure_smoke(repo_root, &binding.retained_row.tool_id).ok();
        rows.push(build_vcf_population_structure_ready_row(
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

    let report = VcfPopulationStructureReadyReport {
        schema_version: VCF_POPULATION_STRUCTURE_READY_SCHEMA_VERSION,
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
    ensure_vcf_population_structure_ready_contract(&report)?;
    Ok(report)
}

fn build_vcf_population_structure_ready_row(
    command_report: &VcfRenderedCommandsReport,
    binding: VcfStageReadinessBinding,
    smoke_report: Option<&LocalVcfPopulationStructureSmokeReport>,
) -> VcfPopulationStructureReadyRow {
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
            && contains_artifact_id(&row.normalized_metrics, "population_structure_report")
            && contains_artifact_id(&row.raw_outputs, "ld_prune_in")
            && contains_artifact_id(&row.raw_outputs, "ld_prune_out")
            && contains_artifact_id(&row.raw_outputs, "population_pca_eigenvec")
            && contains_artifact_id(&row.raw_outputs, "population_pca_eigenval")
            && contains_artifact_id(&row.raw_outputs, "prune_log")
            && contains_artifact_id(&row.raw_outputs, "pca_log")
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
        row.report_section == "population_structure"
            && row.expected_outputs.iter().any(|value| value == "population_structure_report")
            && required_metric_names
                .iter()
                .all(|metric| row.expected_metrics.iter().any(|value| value == metric))
    });
    if !expected_result_ready {
        missing_surfaces.push("vcf_expected_benchmark_results".to_string());
    }

    let report_ready = binding.report_row.as_ref().is_some_and(|row| {
        row.section_id == "population_structure"
            && row.summary_table == "population_structure_metrics"
            && required_metric_names
                .iter()
                .all(|metric| row.metric_columns.iter().any(|value| value == metric))
    });
    if !report_ready {
        missing_surfaces.push("vcf_report_map".to_string());
    }

    let smoke_ready =
        smoke_report.is_some_and(population_structure_smoke_matches_governed_contract);
    if !smoke_ready {
        missing_surfaces.push("local_vcf_population_structure_smoke".to_string());
    }

    let coverage_status = if missing_surfaces.is_empty() {
        COVERAGE_STATUS_COMPLETE.to_string()
    } else {
        COVERAGE_STATUS_INCOMPLETE.to_string()
    };
    let reason = if missing_surfaces.is_empty() {
        format!(
            "`{}` / `{}` stays active with governed adapter, parser, report, and consumed population-structure smoke proof",
            binding.retained_row.stage_id, binding.retained_row.tool_id
        )
    } else {
        format!(
            "`{}` / `{}` is missing readiness surfaces: {}",
            binding.retained_row.stage_id,
            binding.retained_row.tool_id,
            missing_surfaces.join(", ")
        )
    };

    let primary_executables = binding
        .command_row
        .as_ref()
        .map(|row| {
            row.command_steps
                .iter()
                .filter_map(|step| step.argv.first().cloned())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    VcfPopulationStructureReadyRow {
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
        all_domain_active_row_proof_path: binding
            .active_row
            .as_ref()
            .map(|_| "benchmarks/readiness/all-domains/active-stage-tool-matrix.tsv".to_string())
            .unwrap_or_else(|| NO_VALUE.to_string()),
        command_ready,
        command_source: binding
            .command_row
            .as_ref()
            .map(|row| row.command_source.clone())
            .unwrap_or_else(|| NO_VALUE.to_string()),
        command_step_count: binding.command_row.as_ref().map_or(0, |row| row.command_steps.len()),
        command_step_ids: binding
            .command_row
            .as_ref()
            .map(|row| row.command_steps.iter().map(|step| step.step_id.clone()).collect())
            .unwrap_or_default(),
        primary_executables,
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
            .or_else(|| binding.expected_row.as_ref().map(|row| row.report_section.clone()))
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
        smoke_population_structure_json_path: smoke_report
            .map(|report| report.population_structure_json_path.clone())
            .unwrap_or_else(|| NO_VALUE.to_string()),
        smoke_source_population_structure_path: smoke_report
            .map(|report| report.source_population_structure_path.clone())
            .unwrap_or_else(|| NO_VALUE.to_string()),
        smoke_source_pruned_variants_path: smoke_report
            .map(|report| report.source_pruned_variants_path.clone())
            .unwrap_or_else(|| NO_VALUE.to_string()),
        smoke_source_logs_path: smoke_report
            .map(|report| report.source_logs_path.clone())
            .unwrap_or_else(|| NO_VALUE.to_string()),
        smoke_source_pca_report_path: smoke_report
            .map(|report| report.source_pca_report_path.clone())
            .unwrap_or_else(|| NO_VALUE.to_string()),
        smoke_source_admixture_report_path: smoke_report
            .map(|report| report.source_admixture_report_path.clone())
            .unwrap_or_else(|| NO_VALUE.to_string()),
        smoke_stage_result_manifest_path: smoke_report
            .map(|report| report.stage_result_manifest_path.clone())
            .unwrap_or_else(|| NO_VALUE.to_string()),
        smoke_status: smoke_report
            .map(|report| report.status.clone())
            .unwrap_or_else(|| NO_VALUE.to_string()),
        smoke_consumed_pca: smoke_report.map(|report| report.consumed_pca.clone()),
        smoke_consumed_admixture: smoke_report.map(|report| report.consumed_admixture.clone()),
        smoke_sample_group_count: smoke_report.map_or(0, |report| report.sample_groups.len()),
        smoke_sample_groups: smoke_report
            .map(|report| report.sample_groups.clone())
            .unwrap_or_default(),
        smoke_distance_summary: smoke_report.map(|report| report.distance_summary.clone()),
        required_metric_names,
        missing_surfaces,
        coverage_status,
        reason,
    }
}

fn population_structure_smoke_matches_governed_contract(
    report: &LocalVcfPopulationStructureSmokeReport,
) -> bool {
    !report.output_root.trim().is_empty()
        && !report.population_structure_json_path.trim().is_empty()
        && !report.source_population_structure_path.trim().is_empty()
        && !report.source_pruned_variants_path.trim().is_empty()
        && !report.source_logs_path.trim().is_empty()
        && !report.source_pca_report_path.trim().is_empty()
        && !report.source_admixture_report_path.trim().is_empty()
        && !report.stage_result_manifest_path.trim().is_empty()
        && report.status == REQUIRED_STATUS
        && report.consumed_pca.report_path == report.source_pca_report_path
        && report.consumed_pca.sample_count == REQUIRED_SAMPLE_COUNT
        && !report.consumed_pca.execution_mode.trim().is_empty()
        && report.consumed_admixture.report_path == report.source_admixture_report_path
        && report.consumed_admixture.sample_count == REQUIRED_SAMPLE_COUNT
        && report.consumed_admixture.selected_k == REQUIRED_SELECTED_K
        && !report.consumed_admixture.execution_mode.trim().is_empty()
        && report.consumed_admixture.status == REQUIRED_STATUS
        && report.sample_groups.len() == REQUIRED_SAMPLE_IDS.len()
        && report.sample_groups.iter().map(|row| row.sample_id.as_str()).collect::<BTreeSet<_>>()
            == REQUIRED_SAMPLE_IDS.into_iter().collect::<BTreeSet<_>>()
        && report.sample_groups.iter().all(|row| {
            !row.sample_id.trim().is_empty()
                && !row.population_id.trim().is_empty()
                && !row.population_label.trim().is_empty()
                && !row.sex.trim().is_empty()
                && !row.dominant_cluster.trim().is_empty()
                && row.dominant_fraction.is_finite()
                && row.dominant_fraction >= 0.0
                && row.dominant_fraction <= 1.0
                && row.pc1.is_finite()
                && row.pc2.is_finite()
                && row.status == REQUIRED_STATUS
        })
        && report.distance_summary.sample_count == REQUIRED_SAMPLE_COUNT
        && report.distance_summary.pair_count == REQUIRED_PAIR_COUNT
        && report.distance_summary.within_population_pair_count
            == REQUIRED_WITHIN_POPULATION_PAIR_COUNT
        && report.distance_summary.cross_population_pair_count
            == REQUIRED_CROSS_POPULATION_PAIR_COUNT
        && report.distance_summary.min_pc_distance.is_finite()
        && report.distance_summary.max_pc_distance.is_finite()
        && report.distance_summary.mean_pc_distance.is_finite()
        && report.distance_summary.min_pc_distance > 0.0
        && report.distance_summary.max_pc_distance >= report.distance_summary.min_pc_distance
        && report.distance_summary.mean_pc_distance >= report.distance_summary.min_pc_distance
        && report.distance_summary.mean_pc_distance <= report.distance_summary.max_pc_distance
}

fn ensure_vcf_population_structure_ready_contract(
    report: &VcfPopulationStructureReadyReport,
) -> Result<()> {
    if report.retained_row_count != 1
        || report.active_row_count != 1
        || report.complete_row_count != 1
        || report.incomplete_row_count != 0
        || report.violation_count != 0
        || !report.ok
    {
        return Err(anyhow!(
            "vcf.population_structure readiness must retain exactly one complete active governed row"
        ));
    }
    let row = report.rows.first().ok_or_else(|| {
        anyhow!(
            "vcf.population_structure readiness must publish the active governed population-structure row"
        )
    })?;
    if row.stage_id != VCF_POPULATION_STRUCTURE_STAGE_ID
        || row.tool_id != "plink2"
        || row.coverage_status != COVERAGE_STATUS_COMPLETE
        || !row.command_ready
        || !row.output_ready
        || !row.parser_ready
        || !row.expected_result_ready
        || !row.report_ready
        || !row.smoke_ready
    {
        return Err(anyhow!(
            "vcf.population_structure readiness row drifted from the governed active contract"
        ));
    }
    if row.report_section_id != "population_structure"
        || row.summary_table_id != "population_structure_metrics"
        || row.smoke_sample_group_count != REQUIRED_SAMPLE_IDS.len()
    {
        return Err(anyhow!(
            "vcf.population_structure readiness row is missing required report or smoke invariants"
        ));
    }
    Ok(())
}

fn required_metric_names() -> Vec<String> {
    REQUIRED_METRIC_NAMES.iter().map(|metric| (*metric).to_string()).collect()
}

fn contains_artifact_id(values: &[String], artifact_id: &str) -> bool {
    values.iter().any(|value| {
        value
            .split_once('=')
            .is_some_and(|(current_artifact_id, _)| current_artifact_id == artifact_id)
    })
}

fn expected_result_id(
    binding: &super::vcf_expected_benchmark_results::VcfExpectedBenchmarkResultRow,
) -> String {
    format!(
        "vcf:{}:{}:{}:{}",
        binding.corpus_id, binding.stage_id, binding.asset_profile_id, binding.tool_id
    )
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
