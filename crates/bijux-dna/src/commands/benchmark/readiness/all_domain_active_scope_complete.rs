use std::collections::BTreeSet;
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
    render_all_domain_active_stage_tool_matrix, AllDomainActiveStageToolMatrixReport,
    DEFAULT_ALL_DOMAIN_ACTIVE_STAGE_TOOL_MATRIX_PATH,
};
use super::all_domain_adapter_coverage::{
    render_all_domain_adapter_coverage, DEFAULT_ALL_DOMAIN_ADAPTER_COVERAGE_PATH,
};
use super::all_domain_expected_result_coverage::{
    render_all_domain_expected_result_coverage, AllDomainExpectedResultCoverageReport,
    DEFAULT_ALL_DOMAIN_EXPECTED_RESULT_COVERAGE_PATH,
};
use super::all_domain_local_job_coverage::{
    render_all_domain_local_job_coverage, AllDomainLocalJobCoverageReport,
    DEFAULT_ALL_DOMAIN_LOCAL_JOB_COVERAGE_PATH,
};
use super::all_domain_no_declared_only_rows::{
    render_all_domain_no_declared_only_rows, DEFAULT_ALL_DOMAIN_NO_DECLARED_ONLY_ROWS_PATH,
};
use super::all_domain_no_not_benchmark_ready_rows::{
    render_all_domain_no_not_benchmark_ready_rows,
    DEFAULT_ALL_DOMAIN_NO_NOT_BENCHMARK_READY_ROWS_PATH,
};
use super::all_domain_no_placeholder_command_check::{
    render_all_domain_no_placeholder_command_check, AllDomainNoPlaceholderCommandCheckReport,
    DEFAULT_ALL_DOMAIN_NO_PLACEHOLDER_COMMAND_CHECK_PATH,
};
use super::all_domain_no_planned_rows::{
    render_all_domain_no_planned_rows, DEFAULT_ALL_DOMAIN_NO_PLANNED_ROWS_PATH,
};
use super::all_domain_output_contract_coverage::{
    render_all_domain_output_contract_coverage, AllDomainOutputContractCoverageReport,
    DEFAULT_ALL_DOMAIN_OUTPUT_CONTRACT_COVERAGE_PATH,
};
use super::all_domain_parser_fixture_coverage::{
    render_all_domain_parser_fixture_coverage, AllDomainParserFixtureCoverageReport,
    DEFAULT_ALL_DOMAIN_PARSER_FIXTURE_COVERAGE_PATH,
};
use super::all_domain_report_map_coverage::{
    render_all_domain_report_map_coverage, AllDomainReportMapCoverageReport,
    DEFAULT_ALL_DOMAIN_REPORT_MAP_COVERAGE_PATH,
};
use super::all_domain_retained_tools::{
    render_all_domain_retained_tools, DEFAULT_ALL_DOMAIN_RETAINED_TOOLS_PATH,
};
use super::removed_from_scope::{
    render_removed_from_scope, RemovedFromScopeReport, DEFAULT_REMOVED_FROM_SCOPE_PATH,
};
use super::stage_tool_alias_check::{
    render_stage_tool_alias_check, DEFAULT_STAGE_TOOL_ALIAS_CHECK_PATH,
};
use crate::commands::benchmark::active_scope::{
    validate_active_scope_fast, DEFAULT_ACTIVE_SCOPE_FAST_VALIDATION_REPORT_PATH,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_ALL_DOMAIN_ACTIVE_SCOPE_COMPLETE_PATH: &str =
    "benchmarks/readiness/all-domains/ACTIVE_SCOPE_COMPLETE.json";
const ALL_DOMAIN_ACTIVE_SCOPE_COMPLETE_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.all_domain_active_scope_complete.v1";
const COVERAGE_STATUS_COVERED: &str = "covered";
const VCF_IMPUTE_STAGE_ID: &str = "vcf.impute";
const VCF_IMPUTATION_METRICS_STAGE_ID: &str = "vcf.imputation_metrics";
const VCF_LEGACY_IMPUTATION_STAGE_ID: &str = "vcf.imputation";
const VCF_POSTPROCESS_STAGE_ID: &str = "vcf.postprocess";
const VCF_OUTPUT_PARSER_ID: &str = "vcf.parser.vcf_output";
const VCF_REPORT_JSON_PARSER_ID: &str = "vcf.parser.report_json";
const VCF_IMPUTE_SCHEMA_ID: &str = "bijux.schemas.bench.vcf-normalized-metrics.impute.v1";
const VCF_IMPUTATION_METRICS_SCHEMA_ID: &str =
    "bijux.schemas.bench.vcf-normalized-metrics.imputation-metrics.v1";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct BindingKey {
    domain: String,
    stage_id: String,
    tool_id: String,
    corpus_id: String,
    asset_profile_id: String,
}

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
        validate_vcf_postprocess_closure(
            active_stage_tool_matrix.as_ref(),
            removed_from_scope.as_ref(),
            parser_fixture_coverage.as_ref(),
            output_contract_coverage.as_ref(),
            expected_result_coverage.as_ref(),
            report_map_coverage.as_ref(),
            local_job_coverage.as_ref(),
            no_placeholder_command_check.as_ref(),
        ),
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

fn validate_vcf_imputation_identity(
    active_stage_tool_matrix: Option<&AllDomainActiveStageToolMatrixReport>,
    removed_from_scope: Option<&RemovedFromScopeReport>,
) -> Result<String> {
    let active_stage_tool_matrix = active_stage_tool_matrix
        .context("vcf imputation identity check requires the active stage-tool matrix")?;
    let removed_from_scope =
        removed_from_scope.context("vcf imputation identity check requires removed-from-scope")?;

    let active_rows =
        active_stage_tool_matrix.rows.iter().filter(|row| row.domain == "vcf").collect::<Vec<_>>();
    let removed_rows =
        removed_from_scope.rows.iter().filter(|row| row.domain == "vcf").collect::<Vec<_>>();

    let legacy_imputation_row_count = active_rows
        .iter()
        .filter(|row| row.stage_id == VCF_LEGACY_IMPUTATION_STAGE_ID)
        .count()
        + removed_rows.iter().filter(|row| row.stage_id == VCF_LEGACY_IMPUTATION_STAGE_ID).count();
    if legacy_imputation_row_count != 0 {
        return Err(anyhow!(
            "legacy VCF stage id `{VCF_LEGACY_IMPUTATION_STAGE_ID}` is still present in governed active-scope surfaces"
        ));
    }

    let impute_row_count = count_stage_rows(
        active_rows.iter().copied(),
        removed_rows.iter().copied(),
        VCF_IMPUTE_STAGE_ID,
    );
    let imputation_metrics_row_count = count_stage_rows(
        active_rows.iter().copied(),
        removed_rows.iter().copied(),
        VCF_IMPUTATION_METRICS_STAGE_ID,
    );
    if impute_row_count == 0 {
        return Err(anyhow!(
            "normalized VCF stage id `{VCF_IMPUTE_STAGE_ID}` is missing from governed active-scope surfaces"
        ));
    }
    if imputation_metrics_row_count == 0 {
        return Err(anyhow!(
            "normalized VCF stage id `{VCF_IMPUTATION_METRICS_STAGE_ID}` is missing from governed active-scope surfaces"
        ));
    }

    for row in &active_rows {
        if row.stage_id == VCF_IMPUTE_STAGE_ID
            && (row.parser_id != VCF_OUTPUT_PARSER_ID || row.schema_id != VCF_IMPUTE_SCHEMA_ID)
        {
            return Err(anyhow!(
                "`{}` must keep parser `{}` and schema `{}`; found parser `{}` and schema `{}`",
                VCF_IMPUTE_STAGE_ID,
                VCF_OUTPUT_PARSER_ID,
                VCF_IMPUTE_SCHEMA_ID,
                row.parser_id,
                row.schema_id
            ));
        }
        if row.stage_id == VCF_IMPUTATION_METRICS_STAGE_ID
            && (row.parser_id != VCF_REPORT_JSON_PARSER_ID
                || row.schema_id != VCF_IMPUTATION_METRICS_SCHEMA_ID)
        {
            return Err(anyhow!(
                "`{}` must keep parser `{}` and schema `{}`; found parser `{}` and schema `{}`",
                VCF_IMPUTATION_METRICS_STAGE_ID,
                VCF_REPORT_JSON_PARSER_ID,
                VCF_IMPUTATION_METRICS_SCHEMA_ID,
                row.parser_id,
                row.schema_id
            ));
        }
    }
    for row in &removed_rows {
        if row.stage_id == VCF_IMPUTE_STAGE_ID
            && (row.parser_id != VCF_OUTPUT_PARSER_ID || row.schema_id != VCF_IMPUTE_SCHEMA_ID)
        {
            return Err(anyhow!(
                "`{}` must keep parser `{}` and schema `{}`; found parser `{}` and schema `{}`",
                VCF_IMPUTE_STAGE_ID,
                VCF_OUTPUT_PARSER_ID,
                VCF_IMPUTE_SCHEMA_ID,
                row.parser_id,
                row.schema_id
            ));
        }
        if row.stage_id == VCF_IMPUTATION_METRICS_STAGE_ID
            && (row.parser_id != VCF_REPORT_JSON_PARSER_ID
                || row.schema_id != VCF_IMPUTATION_METRICS_SCHEMA_ID)
        {
            return Err(anyhow!(
                "`{}` must keep parser `{}` and schema `{}`; found parser `{}` and schema `{}`",
                VCF_IMPUTATION_METRICS_STAGE_ID,
                VCF_REPORT_JSON_PARSER_ID,
                VCF_IMPUTATION_METRICS_SCHEMA_ID,
                row.parser_id,
                row.schema_id
            ));
        }
    }

    Ok(format!(
        "legacy_imputation_row_count={legacy_imputation_row_count}, impute_row_count={impute_row_count}, imputation_metrics_row_count={imputation_metrics_row_count}"
    ))
}

fn validate_vcf_postprocess_closure(
    active_stage_tool_matrix: Option<&AllDomainActiveStageToolMatrixReport>,
    removed_from_scope: Option<&RemovedFromScopeReport>,
    parser_fixture_coverage: Option<&AllDomainParserFixtureCoverageReport>,
    output_contract_coverage: Option<&AllDomainOutputContractCoverageReport>,
    expected_result_coverage: Option<&AllDomainExpectedResultCoverageReport>,
    report_map_coverage: Option<&AllDomainReportMapCoverageReport>,
    local_job_coverage: Option<&AllDomainLocalJobCoverageReport>,
    no_placeholder_command_check: Option<&AllDomainNoPlaceholderCommandCheckReport>,
) -> Result<String> {
    let active_stage_tool_matrix = active_stage_tool_matrix
        .context("vcf postprocess closure check requires the active stage-tool matrix")?;
    let removed_from_scope =
        removed_from_scope.context("vcf postprocess closure check requires removed-from-scope")?;
    let parser_fixture_coverage = parser_fixture_coverage
        .context("vcf postprocess closure check requires parser fixture coverage")?;
    let output_contract_coverage = output_contract_coverage
        .context("vcf postprocess closure check requires output-contract coverage")?;
    let expected_result_coverage = expected_result_coverage
        .context("vcf postprocess closure check requires expected-result coverage")?;
    let report_map_coverage = report_map_coverage
        .context("vcf postprocess closure check requires report-map coverage")?;
    let local_job_coverage =
        local_job_coverage.context("vcf postprocess closure check requires local-job coverage")?;
    let no_placeholder_command_check = no_placeholder_command_check
        .context("vcf postprocess closure check requires no-placeholder command coverage")?;

    let active_postprocess_keys = active_stage_tool_matrix
        .rows
        .iter()
        .filter(|row| row.domain == "vcf" && row.stage_id == VCF_POSTPROCESS_STAGE_ID)
        .map(binding_key_from_active_row)
        .collect::<BTreeSet<_>>();
    if active_postprocess_keys.is_empty() {
        return Err(anyhow!("`{VCF_POSTPROCESS_STAGE_ID}` must remain inside final active scope"));
    }

    let removed_postprocess_keys = removed_from_scope
        .rows
        .iter()
        .filter(|row| row.domain == "vcf" && row.stage_id == VCF_POSTPROCESS_STAGE_ID)
        .map(binding_key_from_removed_row)
        .collect::<BTreeSet<_>>();
    if !removed_postprocess_keys.is_empty() {
        return Err(anyhow!(
            "`{VCF_POSTPROCESS_STAGE_ID}` cannot be both active and removed from scope"
        ));
    }

    let parser_keys = parser_fixture_coverage
        .rows
        .iter()
        .filter(|row| {
            row.domain == "vcf"
                && row.stage_id == VCF_POSTPROCESS_STAGE_ID
                && row.coverage_status == COVERAGE_STATUS_COVERED
        })
        .map(binding_key_from_parser_row)
        .collect::<BTreeSet<_>>();
    ensure_covering_keys(
        VCF_POSTPROCESS_STAGE_ID,
        "parser fixture coverage",
        &active_postprocess_keys,
        &parser_keys,
    )?;

    let output_keys = output_contract_coverage
        .rows
        .iter()
        .filter(|row| {
            row.domain == "vcf"
                && row.stage_id == VCF_POSTPROCESS_STAGE_ID
                && row.coverage_status == COVERAGE_STATUS_COVERED
        })
        .map(binding_key_from_output_contract_row)
        .collect::<BTreeSet<_>>();
    ensure_covering_keys(
        VCF_POSTPROCESS_STAGE_ID,
        "output-contract coverage",
        &active_postprocess_keys,
        &output_keys,
    )?;

    let expected_keys = expected_result_coverage
        .rows
        .iter()
        .filter(|row| {
            row.domain == "vcf"
                && row.stage_id == VCF_POSTPROCESS_STAGE_ID
                && row.coverage_status == COVERAGE_STATUS_COVERED
        })
        .map(binding_key_from_expected_result_row)
        .collect::<BTreeSet<_>>();
    ensure_covering_keys(
        VCF_POSTPROCESS_STAGE_ID,
        "expected-result coverage",
        &active_postprocess_keys,
        &expected_keys,
    )?;

    let report_keys = report_map_coverage
        .rows
        .iter()
        .filter(|row| {
            row.domain == "vcf"
                && row.stage_id == VCF_POSTPROCESS_STAGE_ID
                && row.coverage_status == COVERAGE_STATUS_COVERED
        })
        .map(binding_key_from_report_map_row)
        .collect::<BTreeSet<_>>();
    ensure_covering_keys(
        VCF_POSTPROCESS_STAGE_ID,
        "report-map coverage",
        &active_postprocess_keys,
        &report_keys,
    )?;

    let local_job_keys = local_job_coverage
        .rows
        .iter()
        .filter(|row| {
            row.domain == "vcf"
                && row.stage_id == VCF_POSTPROCESS_STAGE_ID
                && row.coverage_status == COVERAGE_STATUS_COVERED
        })
        .map(binding_key_from_local_job_row)
        .collect::<BTreeSet<_>>();
    ensure_covering_keys(
        VCF_POSTPROCESS_STAGE_ID,
        "local-job coverage",
        &active_postprocess_keys,
        &local_job_keys,
    )?;

    let command_check_keys = no_placeholder_command_check
        .rows
        .iter()
        .filter(|row| row.domain == "vcf" && row.stage_id == VCF_POSTPROCESS_STAGE_ID && row.ok)
        .map(binding_key_from_no_placeholder_row)
        .collect::<BTreeSet<_>>();
    ensure_covering_keys(
        VCF_POSTPROCESS_STAGE_ID,
        "no-placeholder command check",
        &active_postprocess_keys,
        &command_check_keys,
    )?;

    Ok(format!(
        "active_postprocess_row_count={}, parser_fixture_match_count={}, output_contract_match_count={}, expected_result_match_count={}, report_map_match_count={}, local_job_match_count={}, command_audit_match_count={}",
        active_postprocess_keys.len(),
        parser_keys.len(),
        output_keys.len(),
        expected_keys.len(),
        report_keys.len(),
        local_job_keys.len(),
        command_check_keys.len()
    ))
}

fn ensure_covering_keys(
    stage_id: &str,
    surface_name: &str,
    active_keys: &BTreeSet<BindingKey>,
    covered_keys: &BTreeSet<BindingKey>,
) -> Result<()> {
    let missing_keys =
        active_keys.difference(covered_keys).map(render_binding_key).collect::<Vec<_>>();
    if !missing_keys.is_empty() {
        return Err(anyhow!(
            "`{}` is missing governed {} for {}",
            stage_id,
            surface_name,
            missing_keys.join(", ")
        ));
    }
    Ok(())
}

fn count_stage_rows<'a, I, J>(active_rows: I, removed_rows: J, stage_id: &str) -> usize
where
    I: Iterator<
        Item = &'a super::all_domain_active_stage_tool_matrix::AllDomainActiveStageToolMatrixRow,
    >,
    J: Iterator<Item = &'a super::removed_from_scope::RemovedFromScopeRow>,
{
    active_rows.filter(|row| row.stage_id == stage_id).count()
        + removed_rows.filter(|row| row.stage_id == stage_id).count()
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

fn binding_key_from_active_row(
    row: &super::all_domain_active_stage_tool_matrix::AllDomainActiveStageToolMatrixRow,
) -> BindingKey {
    BindingKey {
        domain: row.domain.clone(),
        stage_id: row.stage_id.clone(),
        tool_id: row.tool_id.clone(),
        corpus_id: row.corpus_id.clone(),
        asset_profile_id: row.asset_profile_id.clone(),
    }
}

fn binding_key_from_removed_row(
    row: &super::removed_from_scope::RemovedFromScopeRow,
) -> BindingKey {
    BindingKey {
        domain: row.domain.clone(),
        stage_id: row.stage_id.clone(),
        tool_id: row.tool_id.clone(),
        corpus_id: row.corpus_id.clone(),
        asset_profile_id: row.asset_profile_id.clone(),
    }
}

fn binding_key_from_parser_row(
    row: &super::all_domain_parser_fixture_coverage::AllDomainParserFixtureCoverageRow,
) -> BindingKey {
    BindingKey {
        domain: row.domain.clone(),
        stage_id: row.stage_id.clone(),
        tool_id: row.tool_id.clone(),
        corpus_id: row.corpus_id.clone(),
        asset_profile_id: row.asset_profile_id.clone(),
    }
}

fn binding_key_from_output_contract_row(
    row: &super::all_domain_output_contract_coverage::AllDomainOutputContractCoverageRow,
) -> BindingKey {
    BindingKey {
        domain: row.domain.clone(),
        stage_id: row.stage_id.clone(),
        tool_id: row.tool_id.clone(),
        corpus_id: row.corpus_id.clone(),
        asset_profile_id: row.asset_profile_id.clone(),
    }
}

fn binding_key_from_expected_result_row(
    row: &super::all_domain_expected_result_coverage::AllDomainExpectedResultCoverageRow,
) -> BindingKey {
    BindingKey {
        domain: row.domain.clone(),
        stage_id: row.stage_id.clone(),
        tool_id: row.tool_id.clone(),
        corpus_id: row.corpus_id.clone(),
        asset_profile_id: row.asset_profile_id.clone(),
    }
}

fn binding_key_from_report_map_row(
    row: &super::all_domain_report_map_coverage::AllDomainReportMapCoverageRow,
) -> BindingKey {
    BindingKey {
        domain: row.domain.clone(),
        stage_id: row.stage_id.clone(),
        tool_id: row.tool_id.clone(),
        corpus_id: row.corpus_id.clone(),
        asset_profile_id: row.asset_profile_id.clone(),
    }
}

fn binding_key_from_local_job_row(
    row: &super::all_domain_local_job_coverage::AllDomainLocalJobCoverageRow,
) -> BindingKey {
    BindingKey {
        domain: row.domain.clone(),
        stage_id: row.stage_id.clone(),
        tool_id: row.tool_id.clone(),
        corpus_id: row.corpus_id.clone(),
        asset_profile_id: row.asset_profile_id.clone(),
    }
}

fn binding_key_from_no_placeholder_row(
    row: &super::all_domain_no_placeholder_command_check::AllDomainNoPlaceholderCommandRow,
) -> BindingKey {
    BindingKey {
        domain: row.domain.clone(),
        stage_id: row.stage_id.clone(),
        tool_id: row.tool_id.clone(),
        corpus_id: row.corpus_id.clone(),
        asset_profile_id: row.asset_profile_id.clone(),
    }
}

fn render_binding_key(key: &BindingKey) -> String {
    format!(
        "{}:{}:{}:{}:{}",
        key.domain, key.stage_id, key.tool_id, key.corpus_id, key.asset_profile_id
    )
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
