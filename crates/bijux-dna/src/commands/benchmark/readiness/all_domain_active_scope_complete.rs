use bijux_dna_api::v1::api::bench::{
    validate_vcf_scope_imputation_identity as validate_vcf_imputation_identity,
    validate_vcf_scope_postprocess_closure as validate_vcf_postprocess_closure,
    VcfScopeCoverageEvidence,
};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use super::all_domain_active_scope_blockers::{
    render_all_domain_active_scope_blockers, DEFAULT_ALL_DOMAIN_ACTIVE_SCOPE_BLOCKERS_PATH,
};
use super::all_domain_active_stage_catalog::{
    render_all_domain_active_stage_catalog, DEFAULT_ALL_DOMAIN_ACTIVE_STAGE_CATALOG_PATH,
};
use super::all_domain_active_stage_tool_matrix::{
    render_all_domain_active_stage_tool_matrix, DEFAULT_ALL_DOMAIN_ACTIVE_STAGE_TOOL_MATRIX_PATH,
};
use super::all_domain_adapter_coverage::{
    render_all_domain_adapter_coverage, DEFAULT_ALL_DOMAIN_ADAPTER_COVERAGE_PATH,
};
use super::all_domain_expected_result_coverage::{
    render_all_domain_expected_result_coverage, DEFAULT_ALL_DOMAIN_EXPECTED_RESULT_COVERAGE_PATH,
};
use super::all_domain_local_job_coverage::{
    render_all_domain_local_job_coverage, DEFAULT_ALL_DOMAIN_LOCAL_JOB_COVERAGE_PATH,
};
use super::all_domain_no_declared_only_rows::{
    render_all_domain_no_declared_only_rows, DEFAULT_ALL_DOMAIN_NO_DECLARED_ONLY_ROWS_PATH,
};
use super::all_domain_no_not_benchmark_ready_rows::{
    render_all_domain_no_not_benchmark_ready_rows,
    DEFAULT_ALL_DOMAIN_NO_NOT_BENCHMARK_READY_ROWS_PATH,
};
use super::all_domain_no_placeholder_command_check::{
    render_all_domain_no_placeholder_command_check,
    DEFAULT_ALL_DOMAIN_NO_PLACEHOLDER_COMMAND_CHECK_PATH,
};
use super::all_domain_no_planned_rows::{
    render_all_domain_no_planned_rows, DEFAULT_ALL_DOMAIN_NO_PLANNED_ROWS_PATH,
};
use super::all_domain_output_contract_coverage::{
    render_all_domain_output_contract_coverage, DEFAULT_ALL_DOMAIN_OUTPUT_CONTRACT_COVERAGE_PATH,
};
use super::all_domain_parser_fixture_coverage::{
    render_all_domain_parser_fixture_coverage, DEFAULT_ALL_DOMAIN_PARSER_FIXTURE_COVERAGE_PATH,
};
use super::all_domain_report_map_coverage::{
    render_all_domain_report_map_coverage, DEFAULT_ALL_DOMAIN_REPORT_MAP_COVERAGE_PATH,
};
use super::all_domain_retained_tools::{
    render_all_domain_retained_tools, DEFAULT_ALL_DOMAIN_RETAINED_TOOLS_PATH,
};
use super::removed_from_scope::{render_removed_from_scope, DEFAULT_REMOVED_FROM_SCOPE_PATH};
use super::stage_tool_alias_check::{
    render_stage_tool_alias_check, DEFAULT_STAGE_TOOL_ALIAS_CHECK_PATH,
};
use crate::commands::benchmark::active_scope::{
    validate_active_scope_fast, DEFAULT_ACTIVE_SCOPE_FAST_VALIDATION_REPORT_PATH,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_ALL_DOMAIN_ACTIVE_SCOPE_COMPLETE_PATH: &str =
    "artifacts/benchmark-smoke/readiness/all-domains/ACTIVE_SCOPE_COMPLETE.json";
const ALL_DOMAIN_ACTIVE_SCOPE_COMPLETE_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.all_domain_active_scope_complete.v1";
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct AllDomainActiveScopeCompleteCheck {
    pub(crate) surface_id: String,
    pub(crate) proof_paths: Vec<String>,
    pub(crate) ok: bool,
    pub(crate) detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AllDomainActiveScopeCompleteReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) active_row_count: usize,
    pub(crate) active_stage_count: usize,
    pub(crate) active_tool_count: usize,
    pub(crate) removed_row_count: usize,
    pub(crate) checked_surface_count: usize,
    pub(crate) passed_surface_count: usize,
    pub(crate) failed_surface_count: usize,
    pub(crate) ok: bool,
    pub(crate) checks: Vec<AllDomainActiveScopeCompleteCheck>,
    pub(crate) failed_checks: Vec<AllDomainActiveScopeCompleteCheck>,
}

pub(crate) fn run_render_all_domain_active_scope_complete(
    args: &parse::BenchReadinessRenderAllDomainActiveScopeCompleteArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_all_domain_active_scope_complete(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_ALL_DOMAIN_ACTIVE_SCOPE_COMPLETE_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_all_domain_active_scope_complete(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<AllDomainActiveScopeCompleteReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let mut checks = Vec::new();

    let retained_tools = record_render_check(
        &mut checks,
        "all_domain_retained_tools",
        &[DEFAULT_ALL_DOMAIN_RETAINED_TOOLS_PATH],
        || {
            render_all_domain_retained_tools(
                repo_root,
                PathBuf::from(DEFAULT_ALL_DOMAIN_RETAINED_TOOLS_PATH),
            )
        },
        |report| {
            format!(
                "row_count={}, active_matrix_tool_count={}, benchmark_ready_tool_count={}",
                report.row_count,
                report.active_matrix_tool_count,
                report.benchmark_ready_tool_count
            )
        },
    );

    let active_stage_catalog = record_render_check(
        &mut checks,
        "all_domain_active_stage_catalog",
        &[DEFAULT_ALL_DOMAIN_ACTIVE_STAGE_CATALOG_PATH],
        || {
            render_all_domain_active_stage_catalog(
                repo_root,
                PathBuf::from(DEFAULT_ALL_DOMAIN_ACTIVE_STAGE_CATALOG_PATH),
            )
        },
        |report| {
            format!(
                "row_count={}, stages_with_benchmark_ready_tools={}, stages_with_report_rows={}",
                report.row_count,
                report.stages_with_benchmark_ready_tools,
                report.stages_with_report_rows
            )
        },
    );

    let active_stage_tool_matrix = record_render_check(
        &mut checks,
        "all_domain_active_stage_tool_matrix",
        &[DEFAULT_ALL_DOMAIN_ACTIVE_STAGE_TOOL_MATRIX_PATH],
        || {
            render_all_domain_active_stage_tool_matrix(
                repo_root,
                PathBuf::from(DEFAULT_ALL_DOMAIN_ACTIVE_STAGE_TOOL_MATRIX_PATH),
            )
        },
        |report| {
            format!(
                "row_count={}, stage_count={}, tool_count={}",
                report.row_count, report.stage_count, report.tool_count
            )
        },
    );

    let no_planned_rows = record_render_check(
        &mut checks,
        "all_domain_no_planned_rows",
        &[DEFAULT_ALL_DOMAIN_NO_PLANNED_ROWS_PATH],
        || {
            render_all_domain_no_planned_rows(
                repo_root,
                PathBuf::from(DEFAULT_ALL_DOMAIN_NO_PLANNED_ROWS_PATH),
            )
        },
        |report| {
            format!(
                "active_row_count={}, removed_row_count={}, violation_count={}",
                report.active_row_count, report.removed_row_count, report.violation_count
            )
        },
    );

    let no_declared_only_rows = record_render_check(
        &mut checks,
        "all_domain_no_declared_only_rows",
        &[DEFAULT_ALL_DOMAIN_NO_DECLARED_ONLY_ROWS_PATH],
        || {
            render_all_domain_no_declared_only_rows(
                repo_root,
                PathBuf::from(DEFAULT_ALL_DOMAIN_NO_DECLARED_ONLY_ROWS_PATH),
            )
        },
        |report| {
            format!(
                "active_row_count={}, removed_row_count={}, violation_count={}",
                report.active_row_count, report.removed_row_count, report.violation_count
            )
        },
    );

    let no_not_benchmark_ready_rows = record_render_check(
        &mut checks,
        "all_domain_no_not_benchmark_ready_rows",
        &[DEFAULT_ALL_DOMAIN_NO_NOT_BENCHMARK_READY_ROWS_PATH],
        || {
            render_all_domain_no_not_benchmark_ready_rows(
                repo_root,
                PathBuf::from(DEFAULT_ALL_DOMAIN_NO_NOT_BENCHMARK_READY_ROWS_PATH),
            )
        },
        |report| {
            format!(
                "active_row_count={}, removed_row_count={}, violation_count={}",
                report.active_row_count, report.removed_row_count, report.violation_count
            )
        },
    );

    let removed_from_scope = record_render_check(
        &mut checks,
        "removed_from_scope",
        &[DEFAULT_REMOVED_FROM_SCOPE_PATH],
        || render_removed_from_scope(repo_root, PathBuf::from(DEFAULT_REMOVED_FROM_SCOPE_PATH)),
        |report| {
            format!(
                "removed_row_count={}, removed_stage_count={}, removed_tool_count={}",
                report.removed_row_count, report.removed_stage_count, report.removed_tool_count
            )
        },
    );

    let stage_tool_alias_check = record_render_check(
        &mut checks,
        "stage_tool_alias_check",
        &[DEFAULT_STAGE_TOOL_ALIAS_CHECK_PATH],
        || {
            render_stage_tool_alias_check(
                repo_root,
                PathBuf::from(DEFAULT_STAGE_TOOL_ALIAS_CHECK_PATH),
            )
        },
        |report| {
            format!(
                "candidate_row_count={}, active_row_count={}, violation_count={}",
                report.candidate_row_count, report.active_row_count, report.violation_count
            )
        },
    );

    let parser_fixture_coverage = record_render_check(
        &mut checks,
        "all_domain_parser_fixture_coverage",
        &[DEFAULT_ALL_DOMAIN_PARSER_FIXTURE_COVERAGE_PATH],
        || {
            render_all_domain_parser_fixture_coverage(
                repo_root,
                PathBuf::from(DEFAULT_ALL_DOMAIN_PARSER_FIXTURE_COVERAGE_PATH),
            )
        },
        |report| {
            format!(
                "covered_row_count={}, missing_row_count={}, coverage_percent={:.1}",
                report.covered_row_count, report.missing_row_count, report.coverage_percent
            )
        },
    );

    let adapter_coverage = record_render_check(
        &mut checks,
        "all_domain_adapter_coverage",
        &[DEFAULT_ALL_DOMAIN_ADAPTER_COVERAGE_PATH],
        || {
            render_all_domain_adapter_coverage(
                repo_root,
                PathBuf::from(DEFAULT_ALL_DOMAIN_ADAPTER_COVERAGE_PATH),
            )
        },
        |report| {
            format!(
                "covered_row_count={}, missing_row_count={}, coverage_percent={:.1}",
                report.covered_row_count, report.missing_row_count, report.coverage_percent
            )
        },
    );

    let output_contract_coverage = record_render_check(
        &mut checks,
        "all_domain_output_contract_coverage",
        &[DEFAULT_ALL_DOMAIN_OUTPUT_CONTRACT_COVERAGE_PATH],
        || {
            render_all_domain_output_contract_coverage(
                repo_root,
                PathBuf::from(DEFAULT_ALL_DOMAIN_OUTPUT_CONTRACT_COVERAGE_PATH),
            )
        },
        |report| {
            format!(
                "covered_row_count={}, missing_row_count={}, index_declared_row_count={}",
                report.covered_row_count, report.missing_row_count, report.index_declared_row_count
            )
        },
    );

    let expected_result_coverage = record_render_check(
        &mut checks,
        "all_domain_expected_result_coverage",
        &[DEFAULT_ALL_DOMAIN_EXPECTED_RESULT_COVERAGE_PATH],
        || {
            render_all_domain_expected_result_coverage(
                repo_root,
                PathBuf::from(DEFAULT_ALL_DOMAIN_EXPECTED_RESULT_COVERAGE_PATH),
            )
        },
        |report| {
            format!(
                "covered_row_count={}, missing_row_count={}, coverage_percent={:.1}",
                report.covered_row_count, report.missing_row_count, report.coverage_percent
            )
        },
    );

    let report_map_coverage = record_render_check(
        &mut checks,
        "all_domain_report_map_coverage",
        &[DEFAULT_ALL_DOMAIN_REPORT_MAP_COVERAGE_PATH],
        || {
            render_all_domain_report_map_coverage(
                repo_root,
                PathBuf::from(DEFAULT_ALL_DOMAIN_REPORT_MAP_COVERAGE_PATH),
            )
        },
        |report| {
            format!(
                "covered_row_count={}, missing_row_count={}, report_map_binding_count={}",
                report.covered_row_count, report.missing_row_count, report.report_map_binding_count
            )
        },
    );

    let local_job_coverage = record_render_check(
        &mut checks,
        "all_domain_local_job_coverage",
        &[DEFAULT_ALL_DOMAIN_LOCAL_JOB_COVERAGE_PATH],
        || {
            render_all_domain_local_job_coverage(
                repo_root,
                PathBuf::from(DEFAULT_ALL_DOMAIN_LOCAL_JOB_COVERAGE_PATH),
            )
        },
        |report| {
            format!(
                "covered_row_count={}, missing_row_count={}, local_job_binding_count={}",
                report.covered_row_count, report.missing_row_count, report.local_job_binding_count
            )
        },
    );

    let active_scope_blockers = record_render_check(
        &mut checks,
        "all_domain_active_scope_blockers",
        &[DEFAULT_ALL_DOMAIN_ACTIVE_SCOPE_BLOCKERS_PATH],
        || {
            render_all_domain_active_scope_blockers(
                repo_root,
                PathBuf::from(DEFAULT_ALL_DOMAIN_ACTIVE_SCOPE_BLOCKERS_PATH),
            )
        },
        |report| {
            format!(
                "row_count={}, blocker_type_count={}, violation_count={}",
                report.row_count,
                report.blocker_type_counts.len(),
                report.violation_count
            )
        },
    );

    let no_placeholder_command_check = record_render_check(
        &mut checks,
        "all_domain_no_placeholder_command_check",
        &[DEFAULT_ALL_DOMAIN_NO_PLACEHOLDER_COMMAND_CHECK_PATH],
        || {
            render_all_domain_no_placeholder_command_check(
                repo_root,
                PathBuf::from(DEFAULT_ALL_DOMAIN_NO_PLACEHOLDER_COMMAND_CHECK_PATH),
            )
        },
        |report| {
            format!(
                "valid_row_count={}, invalid_row_count={}, command_step_count={}",
                report.valid_row_count, report.invalid_row_count, report.command_step_count
            )
        },
    );

    let active_scope_fast_validation = record_render_check(
        &mut checks,
        "active_scope_validate_fast",
        &[DEFAULT_ACTIVE_SCOPE_FAST_VALIDATION_REPORT_PATH],
        || {
            validate_active_scope_fast(
                repo_root,
                PathBuf::from(DEFAULT_ACTIVE_SCOPE_FAST_VALIDATION_REPORT_PATH),
            )
        },
        |report| {
            format!(
                "checked_surface_count={}, failed_surface_count={}",
                report.checked_surface_count, report.failed_surface_count
            )
        },
    );

    record_custom_check(
        &mut checks,
        "vcf_imputation_identity",
        &[DEFAULT_ALL_DOMAIN_ACTIVE_STAGE_TOOL_MATRIX_PATH, DEFAULT_REMOVED_FROM_SCOPE_PATH],
        validate_vcf_imputation_identity(
            active_stage_tool_matrix.as_ref(),
            removed_from_scope.as_ref(),
        ),
    );

    record_custom_check(
        &mut checks,
        "vcf_postprocess_closure",
        &[
            DEFAULT_ALL_DOMAIN_ACTIVE_STAGE_TOOL_MATRIX_PATH,
            DEFAULT_REMOVED_FROM_SCOPE_PATH,
            DEFAULT_ALL_DOMAIN_PARSER_FIXTURE_COVERAGE_PATH,
            DEFAULT_ALL_DOMAIN_OUTPUT_CONTRACT_COVERAGE_PATH,
            DEFAULT_ALL_DOMAIN_EXPECTED_RESULT_COVERAGE_PATH,
            DEFAULT_ALL_DOMAIN_REPORT_MAP_COVERAGE_PATH,
            DEFAULT_ALL_DOMAIN_LOCAL_JOB_COVERAGE_PATH,
            DEFAULT_ALL_DOMAIN_NO_PLACEHOLDER_COMMAND_CHECK_PATH,
        ],
        validate_vcf_postprocess_closure(VcfScopeCoverageEvidence {
            active_stage_tool_matrix: active_stage_tool_matrix.as_ref(),
            removed_from_scope: removed_from_scope.as_ref(),
            parser_fixture_coverage: parser_fixture_coverage.as_ref(),
            output_contract_coverage: output_contract_coverage.as_ref(),
            expected_result_coverage: expected_result_coverage.as_ref(),
            report_map_coverage: report_map_coverage.as_ref(),
            local_job_coverage: local_job_coverage.as_ref(),
            no_placeholder_command_check: no_placeholder_command_check.as_ref(),
        }),
    );

    let checked_surface_count = checks.len();
    let passed_surface_count = checks.iter().filter(|check| check.ok).count();
    let failed_checks = checks.iter().filter(|check| !check.ok).cloned().collect::<Vec<_>>();
    let failed_surface_count = failed_checks.len();

    let active_stage_tool_matrix = active_stage_tool_matrix
        .context("final active-scope gate requires the governed all-domain active matrix")?;
    let removed_from_scope = removed_from_scope
        .context("final active-scope gate requires the removed-from-scope table")?;

    let report = AllDomainActiveScopeCompleteReport {
        schema_version: ALL_DOMAIN_ACTIVE_SCOPE_COMPLETE_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        active_row_count: active_stage_tool_matrix.row_count,
        active_stage_count: active_stage_tool_matrix.stage_count,
        active_tool_count: active_stage_tool_matrix.tool_count,
        removed_row_count: removed_from_scope.removed_row_count,
        checked_surface_count,
        passed_surface_count,
        failed_surface_count,
        ok: failed_surface_count == 0,
        checks,
        failed_checks,
    };

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    bijux_dna_infra::atomic_write_json(&output_path, &report)
        .with_context(|| format!("write {}", output_path.display()))?;
    if !report.ok {
        return Err(anyhow!(
            "all-domain active scope is still ambiguous across {} check(s)",
            report.failed_surface_count
        ));
    }
    let _ = retained_tools;
    let _ = active_stage_catalog;
    let _ = no_planned_rows;
    let _ = no_declared_only_rows;
    let _ = no_not_benchmark_ready_rows;
    let _ = stage_tool_alias_check;
    let _ = adapter_coverage;
    let _ = active_scope_blockers;
    let _ = active_scope_fast_validation;
    Ok(report)
}

fn record_render_check<T, F, S>(
    checks: &mut Vec<AllDomainActiveScopeCompleteCheck>,
    surface_id: &str,
    proof_paths: &[&str],
    render_surface: F,
    summarize: S,
) -> Option<T>
where
    F: FnOnce() -> Result<T>,
    S: FnOnce(&T) -> String,
{
    match render_surface() {
        Ok(report) => {
            checks.push(AllDomainActiveScopeCompleteCheck {
                surface_id: surface_id.to_string(),
                proof_paths: proof_paths.iter().map(|path| (*path).to_string()).collect(),
                ok: true,
                detail: summarize(&report),
            });
            Some(report)
        }
        Err(error) => {
            checks.push(AllDomainActiveScopeCompleteCheck {
                surface_id: surface_id.to_string(),
                proof_paths: proof_paths.iter().map(|path| (*path).to_string()).collect(),
                ok: false,
                detail: error.to_string(),
            });
            None
        }
    }
}

fn record_custom_check(
    checks: &mut Vec<AllDomainActiveScopeCompleteCheck>,
    surface_id: &str,
    proof_paths: &[&str],
    outcome: Result<String>,
) {
    match outcome {
        Ok(detail) => checks.push(AllDomainActiveScopeCompleteCheck {
            surface_id: surface_id.to_string(),
            proof_paths: proof_paths.iter().map(|path| (*path).to_string()).collect(),
            ok: true,
            detail,
        }),
        Err(error) => checks.push(AllDomainActiveScopeCompleteCheck {
            surface_id: surface_id.to_string(),
            proof_paths: proof_paths.iter().map(|path| (*path).to_string()).collect(),
            ok: false,
            detail: error.to_string(),
        }),
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root)
        .map_or_else(|_| path.display().to_string(), |relative| relative.display().to_string())
}

fn repo_relative_path(repo_root: &Path, candidate: &Path) -> PathBuf {
    if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        repo_root.join(candidate)
    }
}
