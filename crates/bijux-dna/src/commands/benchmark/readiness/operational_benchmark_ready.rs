use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use bijux_dna_api::v1::api::bench::{
    build_operational_benchmark_ready_report, evaluate_operational_binding_coverage,
    evaluate_operational_reporting, OperationalBindingEvidence, OperationalReportingEvidence,
    OperationalReportingPaths,
};
pub(crate) use bijux_dna_api::v1::api::bench::{
    OperationalBenchmarkReadyBlocker, OperationalBenchmarkReadyCheck,
    OperationalBenchmarkReadyReport,
};

use super::all_domain_expected_benchmark_results::{
    render_all_domain_expected_benchmark_results,
    DEFAULT_ALL_DOMAIN_EXPECTED_BENCHMARK_RESULTS_PATH,
};
use super::all_domain_local_harness_complete::{
    render_harness_with_dependencies, HarnessReportDependencies,
    DEFAULT_ALL_DOMAIN_LOCAL_HARNESS_COMPLETE_PATH,
};
use super::all_domain_output_declarations::{
    render_all_domain_output_declarations, DEFAULT_ALL_DOMAIN_OUTPUT_DECLARATIONS_PATH,
};
use super::all_domain_parser_collector::{
    render_all_domain_parser_collector, DEFAULT_ALL_DOMAIN_PARSER_COLLECTOR_PATH,
};
use super::all_domain_rendered_commands::{
    render_all_domain_commands, DEFAULT_ALL_DOMAIN_RENDERED_COMMANDS_PATH,
};
use super::all_domain_stage_tool_table::{
    render_all_domain_stage_tool_table, DEFAULT_ALL_DOMAIN_STAGE_TOOL_TABLE_PATH,
};
use super::essential_pipelines_ready::{
    render_essential_pipelines_ready, DEFAULT_ESSENTIAL_PIPELINES_READY_PATH,
};
use super::full_benchmark_dashboard::{
    render_full_benchmark_dashboard_with_dependencies, DashboardReportDependencies,
    DEFAULT_FULL_BENCHMARK_DASHBOARD_MARKDOWN_PATH,
};
use super::full_benchmark_report::{
    render_full_benchmark_report_from_collector, DEFAULT_FULL_BENCHMARK_REPORT_MARKDOWN_PATH,
};
use super::full_benchmark_result_collector::{
    render_full_benchmark_result_collector, DEFAULT_FULL_BENCHMARK_RESULT_COLLECTOR_PATH,
};
use super::stage_tool_resources::{render_stage_tool_resources, DEFAULT_STAGE_TOOL_RESOURCES_PATH};
use super::vcf_adapters_ready::{render_vcf_adapters_ready, DEFAULT_VCF_ADAPTERS_READY_PATH};
use super::vcf_parsers_report_ready::{
    render_vcf_parsers_report_ready, DEFAULT_VCF_PARSERS_REPORT_READY_PATH,
};
use crate::commands::benchmark::local_all_domain_slurm_path_convention::validate_all_domain_slurm_result_paths;
use crate::commands::benchmark::local_all_domain_slurm_script_bodies::DEFAULT_ALL_DOMAIN_SLURM_SCRIPT_BODY_REPORT_PATH;
use crate::commands::benchmark::local_all_domain_slurm_scripts::{
    render_all_domain_slurm_scripts, DEFAULT_ALL_DOMAIN_SLURM_DRY_RUN_ROOT,
};
use crate::commands::benchmark::local_all_domain_slurm_shell_syntax::DEFAULT_ALL_DOMAIN_SLURM_BASH_N_REPORT_PATH;
use crate::commands::benchmark::local_all_domain_slurm_submit_manifest::{
    write_all_domain_slurm_submit_manifest, DEFAULT_ALL_DOMAIN_SLURM_SUBMIT_MANIFEST_PATH,
};
use crate::commands::benchmark::local_slurm_script_bodies::validate_slurm_script_bodies;
use crate::commands::benchmark::local_slurm_shell_syntax::validate_slurm_shell_syntax;
use crate::commands::benchmark::local_vcf_smoke_suite_ready::{
    validate_vcf_smoke_suite_ready, DEFAULT_VCF_SMOKE_SUITE_READY_PATH,
};
use crate::commands::benchmark::local_vcf_stage_catalog_ready::{
    validate_vcf_stage_catalog_ready, DEFAULT_VCF_STAGE_CATALOG_READY_PATH,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_OPERATIONAL_BENCHMARK_READY_PATH: &str =
    "artifacts/benchmark-smoke/readiness/all-domains/FASTQ_BAM_VCF_OPERATIONAL_BENCHMARK_READY.json";

pub(crate) fn run_render_operational_benchmark_ready(
    args: &parse::BenchReadinessRenderOperationalBenchmarkReadyArgs,
) -> Result<()> {
    let repo_root = crate::commands::support::workspace_root::resolve_repo_root()?;
    let report = render_operational_benchmark_ready(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_OPERATIONAL_BENCHMARK_READY_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_operational_benchmark_ready(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<OperationalBenchmarkReadyReport> {
    let absolute_output_path = repo_relative_path(repo_root, &output_path);
    if let Some(parent) = absolute_output_path.parent() {
        std::fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    let mut checks = Vec::new();
    let mut blockers = BTreeSet::<OperationalBenchmarkReadyBlocker>::new();

    let vcf_catalog_ready = match validate_vcf_stage_catalog_ready(
        repo_root,
        PathBuf::from(DEFAULT_VCF_STAGE_CATALOG_READY_PATH),
    ) {
        Ok(report) => {
            checks.push(OperationalBenchmarkReadyCheck {
                surface_id: "vcf_stage_catalog_ready".to_string(),
                output_path: report.output_path.clone(),
                ok: report.ok,
                detail: format!(
                    "checked_goal_count={}, failed_goal_count={}",
                    report.checked_goal_count, report.failed_goal_count
                ),
            });
            Some(report)
        }
        Err(error) => {
            checks.push(failed_check(
                "vcf_stage_catalog_ready",
                DEFAULT_VCF_STAGE_CATALOG_READY_PATH,
                error.to_string(),
            ));
            blockers.insert(global_blocker(
                "vcf",
                "readiness.catalog",
                "catalog",
                "vcf_mini",
                "vcf_catalog",
                "surface_render_failed",
                DEFAULT_VCF_STAGE_CATALOG_READY_PATH,
                "VCF catalog/corpus/truth readiness failed",
            ));
            None
        }
    };

    let vcf_smoke_ready = match validate_vcf_smoke_suite_ready(
        repo_root,
        PathBuf::from(DEFAULT_VCF_SMOKE_SUITE_READY_PATH),
    ) {
        Ok(report) => {
            checks.push(OperationalBenchmarkReadyCheck {
                surface_id: "vcf_smoke_suite_ready".to_string(),
                output_path: report.output_path.clone(),
                ok: report.ok,
                detail: format!(
                    "checked_goal_count={}, failed_goal_count={}",
                    report.checked_goal_count, report.failed_goal_count
                ),
            });
            Some(report)
        }
        Err(error) => {
            checks.push(failed_check(
                "vcf_smoke_suite_ready",
                DEFAULT_VCF_SMOKE_SUITE_READY_PATH,
                error.to_string(),
            ));
            blockers.insert(global_blocker(
                "vcf",
                "readiness.local_smokes",
                "governed_smoke",
                "local_smoke",
                "local_smoke",
                "surface_render_failed",
                DEFAULT_VCF_SMOKE_SUITE_READY_PATH,
                "VCF local smoke suite failed",
            ));
            None
        }
    };

    let vcf_adapters_ready = render_gate_check(
        &mut checks,
        "vcf_adapters_ready",
        DEFAULT_VCF_ADAPTERS_READY_PATH,
        || render_vcf_adapters_ready(repo_root, PathBuf::from(DEFAULT_VCF_ADAPTERS_READY_PATH)),
        |report| {
            report.ok.then_some(format!(
                "benchmark_ready_pair_count={}, rendered_command_pair_count={}",
                report.benchmark_ready_pair_count, report.rendered_command_pair_count
            ))
        },
        |report| report.ok,
    );
    if vcf_adapters_ready.is_none() {
        blockers.insert(global_blocker(
            "vcf",
            "readiness.adapters",
            "governed_adapter",
            "vcf_production_regression",
            "vcf_cohort",
            "surface_render_failed",
            DEFAULT_VCF_ADAPTERS_READY_PATH,
            "VCF adapter readiness failed",
        ));
    }

    let vcf_parsers_ready = render_gate_check(
        &mut checks,
        "vcf_parsers_report_ready",
        DEFAULT_VCF_PARSERS_REPORT_READY_PATH,
        || {
            render_vcf_parsers_report_ready(
                repo_root,
                PathBuf::from(DEFAULT_VCF_PARSERS_REPORT_READY_PATH),
            )
        },
        |report| {
            report.ok.then_some(format!(
                "parser_fixture_row_count={}, active_parser_fixture_row_count={}",
                report.parser_fixture_row_count, report.active_parser_fixture_row_count
            ))
        },
        |report| report.ok,
    );
    if vcf_parsers_ready.is_none() {
        blockers.insert(global_blocker(
            "vcf",
            "readiness.parsers",
            "governed_parser",
            "vcf_production_regression",
            "vcf_cohort",
            "surface_render_failed",
            DEFAULT_VCF_PARSERS_REPORT_READY_PATH,
            "VCF parser/report readiness failed",
        ));
    }

    let expected_results = render_surface(
        &mut checks,
        "all_domain_expected_benchmark_results",
        DEFAULT_ALL_DOMAIN_EXPECTED_BENCHMARK_RESULTS_PATH,
        || {
            render_all_domain_expected_benchmark_results(
                repo_root,
                PathBuf::from(DEFAULT_ALL_DOMAIN_EXPECTED_BENCHMARK_RESULTS_PATH),
            )
        },
        |report| {
            format!("row_count={}, result_id_count={}", report.row_count, report.result_id_count)
        },
    );
    let stage_tool_table = render_surface(
        &mut checks,
        "all_domain_stage_tool_table",
        DEFAULT_ALL_DOMAIN_STAGE_TOOL_TABLE_PATH,
        || {
            render_all_domain_stage_tool_table(
                repo_root,
                PathBuf::from(DEFAULT_ALL_DOMAIN_STAGE_TOOL_TABLE_PATH),
            )
        },
        |report| {
            format!(
                "row_count={}, benchmark_ready_unique_binding_count={}",
                report.row_count, report.benchmark_ready_unique_binding_count
            )
        },
    );
    let rendered_commands = render_surface(
        &mut checks,
        "all_domain_rendered_commands",
        DEFAULT_ALL_DOMAIN_RENDERED_COMMANDS_PATH,
        || {
            render_all_domain_commands(
                repo_root,
                PathBuf::from(DEFAULT_ALL_DOMAIN_RENDERED_COMMANDS_PATH),
            )
        },
        |report| {
            format!("row_count={}, result_id_count={}", report.row_count, report.result_id_count)
        },
    );
    let output_declarations = render_surface(
        &mut checks,
        "all_domain_output_declarations",
        DEFAULT_ALL_DOMAIN_OUTPUT_DECLARATIONS_PATH,
        || {
            render_all_domain_output_declarations(
                repo_root,
                PathBuf::from(DEFAULT_ALL_DOMAIN_OUTPUT_DECLARATIONS_PATH),
            )
        },
        |report| {
            format!(
                "row_count={}, complete_row_count={}",
                report.row_count, report.complete_row_count
            )
        },
    );
    let parser_collector = render_surface(
        &mut checks,
        "all_domain_parser_collector",
        DEFAULT_ALL_DOMAIN_PARSER_COLLECTOR_PATH,
        || {
            render_all_domain_parser_collector(
                repo_root,
                PathBuf::from(DEFAULT_ALL_DOMAIN_PARSER_COLLECTOR_PATH),
            )
        },
        |report| {
            format!(
                "row_count={}, fake_run_row_count={}",
                report.row_count, report.fake_run_row_count
            )
        },
    );
    let resource_report = render_surface(
        &mut checks,
        "stage_tool_resources",
        DEFAULT_STAGE_TOOL_RESOURCES_PATH,
        || render_stage_tool_resources(repo_root, PathBuf::from(DEFAULT_STAGE_TOOL_RESOURCES_PATH)),
        |report| {
            format!(
                "row_count={}, nonzero_resource_row_count={}",
                report.row_count, report.nonzero_resource_row_count
            )
        },
    );
    let essential_pipelines_ready = render_gate_check(
        &mut checks,
        "essential_pipelines_ready",
        DEFAULT_ESSENTIAL_PIPELINES_READY_PATH,
        || {
            render_essential_pipelines_ready(
                repo_root,
                PathBuf::from(DEFAULT_ESSENTIAL_PIPELINES_READY_PATH),
            )
        },
        |report| {
            report.ok.then_some(format!(
                "pipeline_count={}, dag_node_count={}",
                report.pipeline_count, report.dag_node_count
            ))
        },
        |report| report.ok,
    );
    if essential_pipelines_ready.is_none() {
        blockers.insert(global_blocker(
            "cross",
            "pipeline.readiness",
            "pipeline_validation",
            "essential_pipelines",
            "essential_pipelines",
            "surface_render_failed",
            DEFAULT_ESSENTIAL_PIPELINES_READY_PATH,
            "Essential pipeline readiness failed",
        ));
    }

    let all_domain_local_harness_complete = render_gate_check(
        &mut checks,
        "all_domain_local_harness_complete",
        DEFAULT_ALL_DOMAIN_LOCAL_HARNESS_COMPLETE_PATH,
        || {
            render_harness_with_dependencies(
                repo_root,
                PathBuf::from(DEFAULT_ALL_DOMAIN_LOCAL_HARNESS_COMPLETE_PATH),
                Some(HarnessReportDependencies {
                    stage_tools: stage_tool_table.as_ref(),
                    expected_results: expected_results.as_ref(),
                    commands: rendered_commands.as_ref(),
                    outputs: output_declarations.as_ref(),
                    parser: parser_collector.as_ref(),
                }),
            )
        },
        |report| {
            report.ok.then_some(format!(
                "checked_goal_count={}, failed_goal_count={}",
                report.checked_goal_count, report.failed_goal_count
            ))
        },
        |report| report.ok,
    );
    if all_domain_local_harness_complete.is_none() {
        blockers.insert(global_blocker(
            "cross",
            "benchmark.harness",
            "all_domain",
            "benchmark_ready",
            "benchmark_ready",
            "surface_render_failed",
            DEFAULT_ALL_DOMAIN_LOCAL_HARNESS_COMPLETE_PATH,
            "All-domain local harness completion failed",
        ));
    }

    let slurm_scripts = render_surface(
        &mut checks,
        "all_domain_slurm_scripts",
        DEFAULT_ALL_DOMAIN_SLURM_DRY_RUN_ROOT,
        || {
            render_all_domain_slurm_scripts(
                repo_root,
                PathBuf::from(DEFAULT_ALL_DOMAIN_SLURM_DRY_RUN_ROOT),
            )
        },
        |report| {
            format!(
                "script_count={}, benchmark_job_count={}",
                report.script_count, report.benchmark_job_count
            )
        },
    );
    if slurm_scripts.is_none() {
        blockers.insert(global_blocker(
            "cross",
            "slurm.scripts",
            "bijux_dna",
            "benchmark_ready",
            "benchmark_ready",
            "surface_render_failed",
            DEFAULT_ALL_DOMAIN_SLURM_DRY_RUN_ROOT,
            "All-domain SLURM script generation failed",
        ));
    }

    let slurm_body_report = render_surface(
        &mut checks,
        "all_domain_slurm_script_bodies",
        DEFAULT_ALL_DOMAIN_SLURM_SCRIPT_BODY_REPORT_PATH,
        || {
            slurm_scripts
                .as_ref()
                .context("all-domain SLURM scripts must render before body validation")?;
            validate_slurm_script_bodies(
                repo_root,
                PathBuf::from(DEFAULT_ALL_DOMAIN_SLURM_DRY_RUN_ROOT),
                PathBuf::from(DEFAULT_ALL_DOMAIN_SLURM_SCRIPT_BODY_REPORT_PATH),
            )
        },
        |report| {
            format!(
                "script_count={}, findings_count={}",
                report.script_count, report.findings_count
            )
        },
    );
    if slurm_body_report.is_none() {
        blockers.insert(global_blocker(
            "cross",
            "slurm.script_bodies",
            "bijux_dna",
            "benchmark_ready",
            "benchmark_ready",
            "surface_render_failed",
            DEFAULT_ALL_DOMAIN_SLURM_SCRIPT_BODY_REPORT_PATH,
            "All-domain SLURM body validation failed",
        ));
    }

    let slurm_syntax_report = render_surface(
        &mut checks,
        "all_domain_slurm_shell_syntax",
        DEFAULT_ALL_DOMAIN_SLURM_BASH_N_REPORT_PATH,
        || {
            slurm_scripts
                .as_ref()
                .context("all-domain SLURM scripts must render before shell validation")?;
            validate_slurm_shell_syntax(
                repo_root,
                PathBuf::from(DEFAULT_ALL_DOMAIN_SLURM_DRY_RUN_ROOT),
                PathBuf::from(DEFAULT_ALL_DOMAIN_SLURM_BASH_N_REPORT_PATH),
            )
        },
        |report| {
            format!(
                "script_count={}, findings_count={}",
                report.script_count, report.findings_count
            )
        },
    );
    if slurm_syntax_report.is_none() {
        blockers.insert(global_blocker(
            "cross",
            "slurm.shell_syntax",
            "bash",
            "benchmark_ready",
            "benchmark_ready",
            "surface_render_failed",
            DEFAULT_ALL_DOMAIN_SLURM_BASH_N_REPORT_PATH,
            "All-domain SLURM shell syntax validation failed",
        ));
    }

    let slurm_submit_manifest = render_surface(
        &mut checks,
        "all_domain_slurm_submit_manifest",
        DEFAULT_ALL_DOMAIN_SLURM_SUBMIT_MANIFEST_PATH,
        || {
            let scripts = slurm_scripts.as_ref().context(
                "all-domain SLURM scripts must render before submit-manifest construction",
            )?;
            write_all_domain_slurm_submit_manifest(
                repo_root,
                scripts,
                PathBuf::from(DEFAULT_ALL_DOMAIN_SLURM_DRY_RUN_ROOT),
                PathBuf::from(DEFAULT_ALL_DOMAIN_SLURM_SUBMIT_MANIFEST_PATH),
            )
        },
        |report| {
            format!("job_count={}, dependency_count={}", report.job_count, report.dependency_count)
        },
    );
    if slurm_submit_manifest.is_none() {
        blockers.insert(global_blocker(
            "cross",
            "slurm.submit_manifest",
            "bijux_dna",
            "benchmark_ready",
            "benchmark_ready",
            "surface_render_failed",
            DEFAULT_ALL_DOMAIN_SLURM_SUBMIT_MANIFEST_PATH,
            "All-domain SLURM submit manifest failed",
        ));
    }

    let slurm_path_report = render_surface(
        &mut checks,
        "all_domain_slurm_result_paths",
        "artifacts/runs/bench/slurm-dry-run/all-domains/path-convention-check.json",
        || {
            validate_all_domain_slurm_result_paths(
                repo_root,
                PathBuf::from(DEFAULT_ALL_DOMAIN_SLURM_DRY_RUN_ROOT),
                PathBuf::from(DEFAULT_ALL_DOMAIN_SLURM_SUBMIT_MANIFEST_PATH),
                PathBuf::from(
                    "artifacts/runs/bench/slurm-dry-run/all-domains/path-convention-check.json",
                ),
            )
        },
        |report| format!("job_count={}, finding_count={}", report.job_count, report.finding_count),
    );
    if slurm_path_report.is_none() {
        blockers.insert(global_blocker(
            "cross",
            "slurm.result_paths",
            "bijux_dna",
            "benchmark_ready",
            "benchmark_ready",
            "surface_render_failed",
            "artifacts/runs/bench/slurm-dry-run/all-domains/path-convention-check.json",
            "All-domain SLURM result-path validation failed",
        ));
    }

    let result_collector = render_surface(
        &mut checks,
        "full_benchmark_result_collector",
        DEFAULT_FULL_BENCHMARK_RESULT_COLLECTOR_PATH,
        || {
            render_full_benchmark_result_collector(
                repo_root,
                PathBuf::from(DEFAULT_FULL_BENCHMARK_RESULT_COLLECTOR_PATH),
            )
        },
        |report| {
            format!(
            "row_count={}, missing_result_status_count={}, insufficient_data_status_count={}, unsupported_pair_status_count={}",
            report.row_count,
            report.missing_result_status_count,
            report.insufficient_data_status_count,
            report.unsupported_pair_status_count
        )
        },
    );
    let full_report = render_surface(
        &mut checks,
        "full_benchmark_report",
        DEFAULT_FULL_BENCHMARK_REPORT_MARKDOWN_PATH,
        || {
            let dependency = result_collector.as_ref().context(
                "result collector must succeed before full benchmark report construction",
            )?;
            render_full_benchmark_report_from_collector(
                repo_root,
                PathBuf::from(DEFAULT_FULL_BENCHMARK_REPORT_MARKDOWN_PATH),
                dependency,
            )
        },
        |report| {
            format!(
                "row_count={}, missing_result_row_count={}, unsupported_pair_row_count={}",
                report.row_count,
                report.missing_result_row_count,
                report.unsupported_pair_row_count
            )
        },
    );
    let dashboard = render_surface(
        &mut checks,
        "full_benchmark_dashboard",
        DEFAULT_FULL_BENCHMARK_DASHBOARD_MARKDOWN_PATH,
        || {
            let dependency = full_report
                .as_ref()
                .context("full benchmark report must succeed before dashboard construction")?;
            render_full_benchmark_dashboard_with_dependencies(
                repo_root,
                PathBuf::from(DEFAULT_FULL_BENCHMARK_DASHBOARD_MARKDOWN_PATH),
                DashboardReportDependencies {
                    stage_tools: stage_tool_table.as_ref(),
                    expected_results: expected_results.as_ref(),
                    commands: rendered_commands.as_ref(),
                    parser: parser_collector.as_ref(),
                    full_report: Some(dependency),
                },
            )
        },
        |report| {
            format!(
                "total_expected_jobs={}, ready_jobs={}, blocked_jobs={}",
                report.total_expected_jobs, report.ready_jobs, report.blocked_jobs
            )
        },
    );

    checks.push(evaluate_operational_binding_coverage(
        OperationalBindingEvidence {
            expected_results: expected_results.as_ref(),
            stage_tool_table: stage_tool_table.as_ref(),
            rendered_commands: rendered_commands.as_ref(),
            output_declarations: output_declarations.as_ref(),
            parser_collector: parser_collector.as_ref(),
            resource_report: resource_report.as_ref(),
        },
        &mut blockers,
        DEFAULT_ALL_DOMAIN_EXPECTED_BENCHMARK_RESULTS_PATH,
    ));

    let counts = evaluate_operational_reporting(
        OperationalReportingEvidence {
            expected_result_count: expected_results.as_ref().map_or(0, |report| report.row_count),
            collector: result_collector.as_ref(),
            report: full_report.as_ref(),
            dashboard: dashboard.as_ref(),
            vcf_catalog_available: vcf_catalog_ready.is_some(),
            vcf_smoke_available: vcf_smoke_ready.is_some(),
        },
        OperationalReportingPaths {
            collector: DEFAULT_FULL_BENCHMARK_RESULT_COLLECTOR_PATH,
            report: DEFAULT_FULL_BENCHMARK_REPORT_MARKDOWN_PATH,
            dashboard: DEFAULT_FULL_BENCHMARK_DASHBOARD_MARKDOWN_PATH,
            vcf_smoke: DEFAULT_VCF_SMOKE_SUITE_READY_PATH,
        },
        &mut blockers,
    );
    let report = build_operational_benchmark_ready_report(
        path_relative_to_repo(repo_root, &absolute_output_path),
        checks,
        blockers,
        counts,
    );
    bijux_dna_infra::atomic_write_json(&absolute_output_path, &report)?;
    if report.ok {
        Ok(report)
    } else {
        Err(anyhow::anyhow!(
            "operational benchmark readiness gate failed; inspect {}",
            report.output_path
        ))
    }
}

fn render_surface<T, F, D>(
    checks: &mut Vec<OperationalBenchmarkReadyCheck>,
    surface_id: &str,
    output_path: &str,
    render_surface: F,
    detail: D,
) -> Option<T>
where
    F: FnOnce() -> Result<T>,
    D: FnOnce(&T) -> String,
{
    match render_surface() {
        Ok(report) => {
            checks.push(OperationalBenchmarkReadyCheck {
                surface_id: surface_id.to_string(),
                output_path: output_path.to_string(),
                ok: true,
                detail: detail(&report),
            });
            Some(report)
        }
        Err(error) => {
            checks.push(failed_check(surface_id, output_path, error.to_string()));
            None
        }
    }
}

fn render_gate_check<T, F, D, O>(
    checks: &mut Vec<OperationalBenchmarkReadyCheck>,
    surface_id: &str,
    output_path: &str,
    render_surface: F,
    detail: D,
    ok: O,
) -> Option<T>
where
    F: FnOnce() -> Result<T>,
    D: FnOnce(&T) -> Option<String>,
    O: FnOnce(&T) -> bool,
{
    match render_surface() {
        Ok(report) => {
            let is_ok = ok(&report);
            checks.push(OperationalBenchmarkReadyCheck {
                surface_id: surface_id.to_string(),
                output_path: output_path.to_string(),
                ok: is_ok,
                detail: detail(&report)
                    .unwrap_or_else(|| "surface rendered but gate failed".to_string()),
            });
            is_ok.then_some(report)
        }
        Err(error) => {
            checks.push(failed_check(surface_id, output_path, error.to_string()));
            None
        }
    }
}

fn failed_check(
    surface_id: &str,
    output_path: &str,
    detail: String,
) -> OperationalBenchmarkReadyCheck {
    OperationalBenchmarkReadyCheck {
        surface_id: surface_id.to_string(),
        output_path: output_path.to_string(),
        ok: false,
        detail,
    }
}

fn global_blocker(
    domain: &str,
    stage_id: &str,
    tool_id: &str,
    corpus_id: &str,
    asset_profile_id: &str,
    blocker_type: &str,
    blocker_path: &str,
    detail: &str,
) -> OperationalBenchmarkReadyBlocker {
    OperationalBenchmarkReadyBlocker {
        domain: domain.to_string(),
        stage_id: stage_id.to_string(),
        tool_id: tool_id.to_string(),
        corpus_id: corpus_id.to_string(),
        asset_profile_id: asset_profile_id.to_string(),
        blocker_type: blocker_type.to_string(),
        blocker_path: blocker_path.to_string(),
        detail: detail.to_string(),
    }
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
