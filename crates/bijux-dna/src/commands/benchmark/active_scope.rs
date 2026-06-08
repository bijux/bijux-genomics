use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use super::path_resolution::BenchmarkPathResolver;
use super::readiness::all_domain_active_stage_tool_matrix::{
    render_all_domain_active_stage_tool_matrix, DEFAULT_ALL_DOMAIN_ACTIVE_STAGE_TOOL_MATRIX_PATH,
};
use super::readiness::all_domain_adapter_coverage::{
    render_all_domain_adapter_coverage, DEFAULT_ALL_DOMAIN_ADAPTER_COVERAGE_PATH,
};
use super::readiness::all_domain_expected_result_coverage::{
    render_all_domain_expected_result_coverage, DEFAULT_ALL_DOMAIN_EXPECTED_RESULT_COVERAGE_PATH,
};
use super::readiness::all_domain_local_job_coverage::{
    render_all_domain_local_job_coverage, DEFAULT_ALL_DOMAIN_LOCAL_JOB_COVERAGE_PATH,
};
use super::readiness::all_domain_no_placeholder_command_check::{
    render_all_domain_no_placeholder_command_check,
    DEFAULT_ALL_DOMAIN_NO_PLACEHOLDER_COMMAND_CHECK_PATH,
};
use super::readiness::all_domain_output_contract_coverage::{
    render_all_domain_output_contract_coverage, DEFAULT_ALL_DOMAIN_OUTPUT_CONTRACT_COVERAGE_PATH,
};
use super::readiness::all_domain_parser_fixture_coverage::{
    render_all_domain_parser_fixture_coverage, DEFAULT_ALL_DOMAIN_PARSER_FIXTURE_COVERAGE_PATH,
};
use super::readiness::all_domain_report_map_coverage::{
    render_all_domain_report_map_coverage, DEFAULT_ALL_DOMAIN_REPORT_MAP_COVERAGE_PATH,
};
use super::schema_validation::{
    validate_all_domain_schemas, DEFAULT_ALL_DOMAIN_SCHEMA_VALIDATION_REPORT_PATH,
};
use crate::commands::cli::parse::{self, BenchSchemaDomainArg};
use crate::commands::cli::render;
use crate::commands::fixtures::paths::benchmark_fixture_root_path;
use crate::commands::fixtures::root_validation::{
    validate_benchmark_fixture_root, DEFAULT_BENCHMARK_FIXTURE_ROOT_VALIDATION_REPORT_PATH,
};

pub(crate) const DEFAULT_ACTIVE_SCOPE_FAST_VALIDATION_REPORT_PATH: &str =
    "artifacts/bench-active-scope/validate-fast.json";
const ACTIVE_SCOPE_FAST_VALIDATION_SCHEMA_VERSION: &str =
    "bijux.bench.active_scope_validate_fast.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct ActiveScopeFastValidationCheck {
    pub(crate) category: String,
    pub(crate) surface_id: String,
    pub(crate) output_path: String,
    pub(crate) ok: bool,
    pub(crate) detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ActiveScopeFastValidationReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) mode: String,
    pub(crate) benchmark_root: String,
    pub(crate) schema_root: String,
    pub(crate) fixture_root: String,
    pub(crate) checked_surface_count: usize,
    pub(crate) passed_surface_count: usize,
    pub(crate) failed_surface_count: usize,
    pub(crate) category_counts: BTreeMap<String, usize>,
    pub(crate) failed_category_counts: BTreeMap<String, usize>,
    pub(crate) ok: bool,
    pub(crate) checks: Vec<ActiveScopeFastValidationCheck>,
}

pub(crate) fn run_active_scope_validate_command(
    repo_root: &Path,
    args: &parse::BenchActiveScopeValidateArgs,
) -> Result<()> {
    if !args.fast {
        return Err(anyhow!("bench active-scope validate currently requires `--fast`"));
    }
    let report = validate_active_scope_fast(
        repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_ACTIVE_SCOPE_FAST_VALIDATION_REPORT_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn validate_active_scope_fast(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<ActiveScopeFastValidationReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let resolver = BenchmarkPathResolver::new(repo_root, None);
    let schema_root = resolver.benchmark_schema_root();
    let fixture_root = benchmark_fixture_root_path(repo_root, None);
    let mut checks = Vec::new();

    record_check(
        &mut checks,
        "configs",
        "all_domain_active_stage_tool_matrix",
        DEFAULT_ALL_DOMAIN_ACTIVE_STAGE_TOOL_MATRIX_PATH,
        || {
            render_all_domain_active_stage_tool_matrix(
                repo_root,
                PathBuf::from(DEFAULT_ALL_DOMAIN_ACTIVE_STAGE_TOOL_MATRIX_PATH),
            )
        },
        |report| {
            (
                report.output_path.clone(),
                format!(
                    "row_count={}, stage_count={}, tool_count={}",
                    report.row_count, report.stage_count, report.tool_count
                ),
            )
        },
    );

    record_check(
        &mut checks,
        "schemas",
        "all_domain_schema_validation",
        DEFAULT_ALL_DOMAIN_SCHEMA_VALIDATION_REPORT_PATH,
        || {
            validate_all_domain_schemas(
                repo_root,
                PathBuf::from(DEFAULT_ALL_DOMAIN_SCHEMA_VALIDATION_REPORT_PATH),
                &schema_root,
                &[
                    BenchSchemaDomainArg::Fastq,
                    BenchSchemaDomainArg::Bam,
                    BenchSchemaDomainArg::Vcf,
                ],
            )
        },
        |report| {
            (
                report.output_path.clone(),
                format!(
                    "domain_count={}, passed_domain_count={}, failed_domain_count={}",
                    report.domain_count, report.passed_domain_count, report.failed_domain_count
                ),
            )
        },
    );

    record_check(
        &mut checks,
        "fixtures",
        "benchmark_fixture_root_validation",
        DEFAULT_BENCHMARK_FIXTURE_ROOT_VALIDATION_REPORT_PATH,
        || {
            validate_benchmark_fixture_root(
                repo_root,
                &fixture_root,
                PathBuf::from(DEFAULT_BENCHMARK_FIXTURE_ROOT_VALIDATION_REPORT_PATH),
            )
        },
        |report| {
            (
                report.output_path.clone(),
                format!(
                    "checked_fixture_count={}, invalid_fixture_count={}",
                    report.checked_fixture_count, report.invalid_fixture_count
                ),
            )
        },
    );

    record_check(
        &mut checks,
        "adapters",
        "all_domain_adapter_coverage",
        DEFAULT_ALL_DOMAIN_ADAPTER_COVERAGE_PATH,
        || {
            render_all_domain_adapter_coverage(
                repo_root,
                PathBuf::from(DEFAULT_ALL_DOMAIN_ADAPTER_COVERAGE_PATH),
            )
        },
        |report| {
            (
                report.output_path.clone(),
                format!(
                    "covered_row_count={}, missing_row_count={}",
                    report.covered_row_count, report.missing_row_count
                ),
            )
        },
    );

    record_check(
        &mut checks,
        "commands",
        "all_domain_local_job_coverage",
        DEFAULT_ALL_DOMAIN_LOCAL_JOB_COVERAGE_PATH,
        || {
            render_all_domain_local_job_coverage(
                repo_root,
                PathBuf::from(DEFAULT_ALL_DOMAIN_LOCAL_JOB_COVERAGE_PATH),
            )
        },
        |report| {
            (
                report.output_path.clone(),
                format!(
                    "covered_row_count={}, missing_row_count={}",
                    report.covered_row_count, report.missing_row_count
                ),
            )
        },
    );

    record_check(
        &mut checks,
        "commands",
        "all_domain_no_placeholder_command_check",
        DEFAULT_ALL_DOMAIN_NO_PLACEHOLDER_COMMAND_CHECK_PATH,
        || {
            render_all_domain_no_placeholder_command_check(
                repo_root,
                PathBuf::from(DEFAULT_ALL_DOMAIN_NO_PLACEHOLDER_COMMAND_CHECK_PATH),
            )
        },
        |report| {
            (
                report.output_path.clone(),
                format!(
                    "valid_row_count={}, invalid_row_count={}, violation_count={}",
                    report.valid_row_count, report.invalid_row_count, report.violation_count
                ),
            )
        },
    );

    record_check(
        &mut checks,
        "parsers",
        "all_domain_parser_fixture_coverage",
        DEFAULT_ALL_DOMAIN_PARSER_FIXTURE_COVERAGE_PATH,
        || {
            render_all_domain_parser_fixture_coverage(
                repo_root,
                PathBuf::from(DEFAULT_ALL_DOMAIN_PARSER_FIXTURE_COVERAGE_PATH),
            )
        },
        |report| {
            (
                report.output_path.clone(),
                format!(
                    "covered_row_count={}, missing_row_count={}",
                    report.covered_row_count, report.missing_row_count
                ),
            )
        },
    );

    record_check(
        &mut checks,
        "outputs",
        "all_domain_output_contract_coverage",
        DEFAULT_ALL_DOMAIN_OUTPUT_CONTRACT_COVERAGE_PATH,
        || {
            render_all_domain_output_contract_coverage(
                repo_root,
                PathBuf::from(DEFAULT_ALL_DOMAIN_OUTPUT_CONTRACT_COVERAGE_PATH),
            )
        },
        |report| {
            (
                report.output_path.clone(),
                format!(
                    "covered_row_count={}, missing_row_count={}",
                    report.covered_row_count, report.missing_row_count
                ),
            )
        },
    );

    record_check(
        &mut checks,
        "expected_results",
        "all_domain_expected_result_coverage",
        DEFAULT_ALL_DOMAIN_EXPECTED_RESULT_COVERAGE_PATH,
        || {
            render_all_domain_expected_result_coverage(
                repo_root,
                PathBuf::from(DEFAULT_ALL_DOMAIN_EXPECTED_RESULT_COVERAGE_PATH),
            )
        },
        |report| {
            (
                report.output_path.clone(),
                format!(
                    "covered_row_count={}, missing_row_count={}",
                    report.covered_row_count, report.missing_row_count
                ),
            )
        },
    );

    record_check(
        &mut checks,
        "reports",
        "all_domain_report_map_coverage",
        DEFAULT_ALL_DOMAIN_REPORT_MAP_COVERAGE_PATH,
        || {
            render_all_domain_report_map_coverage(
                repo_root,
                PathBuf::from(DEFAULT_ALL_DOMAIN_REPORT_MAP_COVERAGE_PATH),
            )
        },
        |report| {
            (
                report.output_path.clone(),
                format!(
                    "covered_row_count={}, missing_row_count={}",
                    report.covered_row_count, report.missing_row_count
                ),
            )
        },
    );

    let checked_surface_count = checks.len();
    let passed_surface_count = checks.iter().filter(|check| check.ok).count();
    let failed_surface_count = checked_surface_count.saturating_sub(passed_surface_count);
    let mut category_counts = BTreeMap::<String, usize>::new();
    let mut failed_category_counts = BTreeMap::<String, usize>::new();
    for check in &checks {
        *category_counts.entry(check.category.clone()).or_default() += 1;
        if !check.ok {
            *failed_category_counts.entry(check.category.clone()).or_default() += 1;
        }
    }

    let report = ActiveScopeFastValidationReport {
        schema_version: ACTIVE_SCOPE_FAST_VALIDATION_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        mode: "fast".to_string(),
        benchmark_root: path_relative_to_repo(repo_root, resolver.benchmark_root()),
        schema_root: path_relative_to_repo(repo_root, &schema_root),
        fixture_root: path_relative_to_repo(repo_root, &fixture_root),
        checked_surface_count,
        passed_surface_count,
        failed_surface_count,
        category_counts,
        failed_category_counts,
        ok: failed_surface_count == 0,
        checks,
    };

    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    bijux_dna_infra::atomic_write_json(&output_path, &report)
        .with_context(|| format!("write {}", output_path.display()))?;

    if !report.ok {
        return Err(anyhow!(
            "active-scope fast validation failed across {} surface(s)",
            report.failed_surface_count
        ));
    }
    Ok(report)
}

fn record_check<T, F, S>(
    checks: &mut Vec<ActiveScopeFastValidationCheck>,
    category: &str,
    surface_id: &str,
    fallback_output_path: &str,
    render_surface: F,
    summarize: S,
) where
    F: FnOnce() -> Result<T>,
    S: FnOnce(&T) -> (String, String),
{
    match render_surface() {
        Ok(report) => {
            let (output_path, detail) = summarize(&report);
            checks.push(ActiveScopeFastValidationCheck {
                category: category.to_string(),
                surface_id: surface_id.to_string(),
                output_path,
                ok: true,
                detail,
            });
        }
        Err(error) => {
            checks.push(ActiveScopeFastValidationCheck {
                category: category.to_string(),
                surface_id: surface_id.to_string(),
                output_path: fallback_output_path.to_string(),
                ok: false,
                detail: error.to_string(),
            });
        }
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root)
        .map(|relative| relative.display().to_string())
        .unwrap_or_else(|_| path.display().to_string())
}

fn repo_relative_path(repo_root: &Path, candidate: &Path) -> PathBuf {
    if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        repo_root.join(candidate)
    }
}
