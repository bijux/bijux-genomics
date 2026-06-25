use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use super::all_domain_active_stage_tool_matrix::{
    render_all_domain_active_stage_tool_matrix, DEFAULT_ALL_DOMAIN_ACTIVE_STAGE_TOOL_MATRIX_PATH,
};
use super::bam_adapter_output_contract::{
    render_bam_adapter_output_contract, BamAdapterOutputContractReport,
    DEFAULT_BAM_ADAPTER_OUTPUT_CONTRACT_PATH,
};
use super::bam_parser_fixture_coverage::{
    render_bam_parser_fixture_coverage, DEFAULT_BAM_PARSER_FIXTURE_COVERAGE_PATH,
};
use super::bam_rendered_commands::{render_bam_commands, DEFAULT_BAM_RENDERED_COMMANDS_PATH};
use super::bam_report_map::{render_bam_report_map, DEFAULT_BAM_REPORT_MAP_PATH};
use super::expected_benchmark_results::{
    render_expected_benchmark_results, ExpectedBenchmarkResultsReport,
    DEFAULT_EXPECTED_BENCHMARK_RESULTS_PATH,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_BAM_ACTIVE_ROW_CONSISTENCY_PATH: &str =
    "benchmarks/readiness/bam/BAM_ACTIVE_ROW_CONSISTENCY.json";
const BAM_ACTIVE_ROW_CONSISTENCY_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.bam_active_row_consistency.v1";
const BAM_DOMAIN: &str = "bam";
const CHECKED_SURFACE_COUNT: usize = 6;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct BamBindingKey {
    stage_id: String,
    tool_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct BamActiveRowConsistencySurfaceCheck {
    pub(crate) surface_id: String,
    pub(crate) output_path: String,
    pub(crate) row_count: usize,
    pub(crate) stage_count: usize,
    pub(crate) tool_count: usize,
    pub(crate) binding_count: usize,
    pub(crate) ok: bool,
    pub(crate) missing_bindings: Vec<String>,
    pub(crate) extra_bindings: Vec<String>,
    pub(crate) detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BamActiveRowConsistencyReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) active_matrix_output_path: String,
    pub(crate) active_row_count: usize,
    pub(crate) active_stage_count: usize,
    pub(crate) active_tool_count: usize,
    pub(crate) checked_surface_count: usize,
    pub(crate) passed_surface_count: usize,
    pub(crate) failed_surface_count: usize,
    pub(crate) ok: bool,
    pub(crate) surfaces: Vec<BamActiveRowConsistencySurfaceCheck>,
    pub(crate) failed_surfaces: Vec<BamActiveRowConsistencySurfaceCheck>,
}

pub(crate) fn run_render_bam_active_row_consistency(
    args: &parse::BenchReadinessRenderBamActiveRowConsistencyArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_bam_active_row_consistency(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_BAM_ACTIVE_ROW_CONSISTENCY_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_bam_active_row_consistency(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<BamActiveRowConsistencyReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    let report = build_bam_active_row_consistency_report(repo_root, &output_path)?;
    bijux_dna_infra::atomic_write_json(&output_path, &report)?;
    if !report.ok {
        return Err(anyhow!(
            "bam active rows must stay count-consistent across rendered commands, output declarations, expected results, parser fixtures, local jobs, and report-map rows"
        ));
    }
    Ok(report)
}

fn build_bam_active_row_consistency_report(
    repo_root: &Path,
    output_path: &Path,
) -> Result<BamActiveRowConsistencyReport> {
    let active_matrix = render_all_domain_active_stage_tool_matrix(
        repo_root,
        PathBuf::from(DEFAULT_ALL_DOMAIN_ACTIVE_STAGE_TOOL_MATRIX_PATH),
    )?;
    let rendered_commands =
        render_bam_commands(repo_root, PathBuf::from(DEFAULT_BAM_RENDERED_COMMANDS_PATH))?;
    let output_declarations = render_bam_adapter_output_contract(
        repo_root,
        PathBuf::from(DEFAULT_BAM_ADAPTER_OUTPUT_CONTRACT_PATH),
    )?;
    let expected_results = render_expected_benchmark_results(
        repo_root,
        PathBuf::from(DEFAULT_EXPECTED_BENCHMARK_RESULTS_PATH),
    )?;
    let parser_fixtures = render_bam_parser_fixture_coverage(
        repo_root,
        PathBuf::from(DEFAULT_BAM_PARSER_FIXTURE_COVERAGE_PATH),
    )?;
    let report_map = render_bam_report_map(repo_root, PathBuf::from(DEFAULT_BAM_REPORT_MAP_PATH))?;

    let active_rows =
        active_matrix.rows.iter().filter(|row| row.domain == BAM_DOMAIN).collect::<Vec<_>>();
    let active_bindings = active_rows
        .iter()
        .map(|row| binding_key(&row.stage_id, &row.tool_id))
        .collect::<BTreeSet<_>>();
    let active_stage_count =
        active_rows.iter().map(|row| row.stage_id.as_str()).collect::<BTreeSet<_>>().len();
    let active_tool_count =
        active_rows.iter().map(|row| row.tool_id.as_str()).collect::<BTreeSet<_>>().len();

    let surfaces = vec![
        build_surface_check(
            "bam_rendered_commands",
            &rendered_commands.output_path,
            rendered_commands.row_count,
            rendered_commands.stage_count,
            rendered_commands.tool_count,
            rendered_commands
                .rows
                .iter()
                .map(|row| binding_key(&row.stage_id, &row.tool_id))
                .collect::<BTreeSet<_>>(),
            &active_bindings,
            "validated one BAM rendered command row per active BAM binding",
        ),
        build_surface_check(
            "bam_output_declarations",
            &output_declarations.output_path,
            output_declarations.row_count,
            count_stage_ids_from_output_declarations(&output_declarations),
            count_tool_ids_from_output_declarations(&output_declarations),
            output_declarations
                .rows
                .iter()
                .map(|row| binding_key(&row.stage_id, &row.tool_id))
                .collect::<BTreeSet<_>>(),
            &active_bindings,
            "validated one BAM output-declaration row per active BAM binding",
        ),
        build_surface_check(
            "bam_expected_results",
            &expected_results.output_path,
            expected_results.rows.iter().filter(|row| row.domain == BAM_DOMAIN).count(),
            count_bam_stage_ids_from_expected_results(&expected_results),
            count_bam_tool_ids_from_expected_results(&expected_results),
            expected_results
                .rows
                .iter()
                .filter(|row| row.domain == BAM_DOMAIN)
                .map(|row| binding_key(&row.stage_id, &row.tool_id))
                .collect::<BTreeSet<_>>(),
            &active_bindings,
            "validated one BAM expected-result row per active BAM binding",
        ),
        build_surface_check(
            "bam_parser_fixtures",
            &parser_fixtures.output_path,
            parser_fixtures.row_count,
            parser_fixtures.stage_count,
            parser_fixtures.tool_count,
            parser_fixtures
                .rows
                .iter()
                .map(|row| binding_key(&row.stage_id, &row.tool_id))
                .collect::<BTreeSet<_>>(),
            &active_bindings,
            "validated one BAM parser-fixture row per active BAM binding",
        ),
        build_surface_check(
            "bam_local_jobs",
            &rendered_commands.argv_output_path,
            rendered_commands.row_count,
            rendered_commands.stage_count,
            rendered_commands.tool_count,
            rendered_commands
                .rows
                .iter()
                .map(|row| binding_key(&row.stage_id, &row.tool_id))
                .collect::<BTreeSet<_>>(),
            &active_bindings,
            "validated one BAM local-job row per active BAM binding",
        ),
        build_surface_check(
            "bam_report_map",
            &report_map.output_path,
            report_map.row_count,
            report_map.stage_count,
            report_map.tool_count,
            report_map
                .rows
                .iter()
                .map(|row| binding_key(&row.stage_id, &row.tool_id))
                .collect::<BTreeSet<_>>(),
            &active_bindings,
            "validated one BAM report-map row per active BAM binding",
        ),
    ];

    let passed_surface_count = surfaces.iter().filter(|surface| surface.ok).count();
    let failed_surfaces =
        surfaces.iter().filter(|surface| !surface.ok).cloned().collect::<Vec<_>>();
    let failed_surface_count = failed_surfaces.len();

    Ok(BamActiveRowConsistencyReport {
        schema_version: BAM_ACTIVE_ROW_CONSISTENCY_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, output_path),
        active_matrix_output_path: active_matrix.output_path,
        active_row_count: active_rows.len(),
        active_stage_count,
        active_tool_count,
        checked_surface_count: surfaces.len(),
        passed_surface_count,
        failed_surface_count,
        ok: failed_surface_count == 0
            && surfaces.len() == CHECKED_SURFACE_COUNT
            && active_rows.len() == active_bindings.len(),
        surfaces,
        failed_surfaces,
    })
}

fn build_surface_check(
    surface_id: &str,
    output_path: &str,
    row_count: usize,
    stage_count: usize,
    tool_count: usize,
    surface_bindings: BTreeSet<BamBindingKey>,
    active_bindings: &BTreeSet<BamBindingKey>,
    success_detail: &str,
) -> BamActiveRowConsistencySurfaceCheck {
    let binding_count = surface_bindings.len();
    let missing_bindings = diff_bindings(active_bindings, &surface_bindings);
    let extra_bindings = diff_bindings(&surface_bindings, active_bindings);
    let ok = row_count == active_bindings.len()
        && binding_count == active_bindings.len()
        && missing_bindings.is_empty()
        && extra_bindings.is_empty();
    let detail = if ok {
        success_detail.to_string()
    } else {
        format!(
            "surface `{surface_id}` drifted from active BAM rows: rows={row_count}, bindings={binding_count}, missing={}, extra={}",
            join_or_none(&missing_bindings),
            join_or_none(&extra_bindings),
        )
    };

    BamActiveRowConsistencySurfaceCheck {
        surface_id: surface_id.to_string(),
        output_path: output_path.to_string(),
        row_count,
        stage_count,
        tool_count,
        binding_count,
        ok,
        missing_bindings,
        extra_bindings,
        detail,
    }
}

fn binding_key(stage_id: &str, tool_id: &str) -> BamBindingKey {
    BamBindingKey { stage_id: stage_id.to_string(), tool_id: tool_id.to_string() }
}

fn diff_bindings(left: &BTreeSet<BamBindingKey>, right: &BTreeSet<BamBindingKey>) -> Vec<String> {
    left.difference(right)
        .map(|binding| format!("{}/{}", binding.stage_id, binding.tool_id))
        .collect::<Vec<_>>()
}

fn join_or_none(values: &[String]) -> String {
    if values.is_empty() {
        "none".to_string()
    } else {
        values.join(", ")
    }
}

fn count_stage_ids_from_output_declarations(report: &BamAdapterOutputContractReport) -> usize {
    report.rows.iter().map(|row| row.stage_id.as_str()).collect::<BTreeSet<_>>().len()
}

fn count_tool_ids_from_output_declarations(report: &BamAdapterOutputContractReport) -> usize {
    report.rows.iter().map(|row| row.tool_id.as_str()).collect::<BTreeSet<_>>().len()
}

fn count_bam_stage_ids_from_expected_results(report: &ExpectedBenchmarkResultsReport) -> usize {
    report
        .rows
        .iter()
        .filter(|row| row.domain == BAM_DOMAIN)
        .map(|row| row.stage_id.as_str())
        .collect::<BTreeSet<_>>()
        .len()
}

fn count_bam_tool_ids_from_expected_results(report: &ExpectedBenchmarkResultsReport) -> usize {
    report
        .rows
        .iter()
        .filter(|row| row.domain == BAM_DOMAIN)
        .map(|row| row.tool_id.as_str())
        .collect::<BTreeSet<_>>()
        .len()
}

fn repo_relative_path(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.strip_prefix(repo_root).unwrap_or(path).to_path_buf()
    } else {
        path.to_path_buf()
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    repo_relative_path(repo_root, path).display().to_string()
}
