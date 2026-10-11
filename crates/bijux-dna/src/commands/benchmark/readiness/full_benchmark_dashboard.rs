use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
pub(crate) use bijux_dna_api::v1::api::bench::FullBenchmarkDashboardReport;
use bijux_dna_api::v1::api::bench::{
    evaluate_full_benchmark_dashboard, render_full_benchmark_dashboard_markdown,
    FullBenchmarkDashboardEvidence, FullBenchmarkDashboardPaths,
};

use super::all_domain_expected_benchmark_results::{
    render_all_domain_expected_benchmark_results,
    DEFAULT_ALL_DOMAIN_EXPECTED_BENCHMARK_RESULTS_PATH,
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
use super::full_benchmark_report::{
    render_full_benchmark_report, FullBenchmarkReport, DEFAULT_FULL_BENCHMARK_REPORT_MARKDOWN_PATH,
};
use crate::commands::benchmark::local_real_smoke_core_subset::{
    render_real_smoke_core_subset, DEFAULT_REAL_SMOKE_CORE_SUBSET_PATH,
};
use crate::commands::benchmark::local_stage_inventory::{
    render_all_domain_stage_inventory, BenchLocalDomain, DEFAULT_ALL_DOMAIN_STAGE_LIST_PATH,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_FULL_BENCHMARK_DASHBOARD_MARKDOWN_PATH: &str =
    "artifacts/benchmark-smoke/readiness/all-domains/FASTQ_BAM_VCF_BENCHMARK_DASHBOARD.md";
pub(crate) const DEFAULT_FULL_BENCHMARK_DASHBOARD_JSON_PATH: &str =
    "artifacts/benchmark-smoke/readiness/all-domains/FASTQ_BAM_VCF_BENCHMARK_DASHBOARD.json";

pub(crate) fn run_render_full_benchmark_dashboard(
    args: &parse::BenchReadinessRenderFullBenchmarkDashboardArgs,
) -> Result<()> {
    let repo_root = crate::commands::support::workspace_root::resolve_repo_root()?;
    let report = render_full_benchmark_dashboard(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_FULL_BENCHMARK_DASHBOARD_MARKDOWN_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.markdown_output_path);
    }
    Ok(())
}

pub(crate) fn render_full_benchmark_dashboard(
    repo_root: &Path,
    markdown_output_path: PathBuf,
) -> Result<FullBenchmarkDashboardReport> {
    render_full_benchmark_dashboard_with_dependency(repo_root, markdown_output_path, None)
}

/// Uses the successful upstream observation from this readiness request.
/// The caller must keep its repository inputs and referenced artifacts alive and unchanged.
pub(crate) fn render_full_benchmark_dashboard_from_report(
    repo_root: &Path,
    markdown_output_path: PathBuf,
    dependency: &FullBenchmarkReport,
) -> Result<FullBenchmarkDashboardReport> {
    render_full_benchmark_dashboard_with_dependency(
        repo_root,
        markdown_output_path,
        Some(dependency),
    )
}

fn render_full_benchmark_dashboard_with_dependency(
    repo_root: &Path,
    markdown_output_path: PathBuf,
    dependency: Option<&FullBenchmarkReport>,
) -> Result<FullBenchmarkDashboardReport> {
    let markdown_output_path = repo_relative_path(repo_root, &markdown_output_path);
    let json_output_path = derive_json_output_path(&markdown_output_path);

    let stage_inventory = render_all_domain_stage_inventory(
        repo_root,
        &[BenchLocalDomain::Fastq, BenchLocalDomain::Bam, BenchLocalDomain::Vcf],
        PathBuf::from(DEFAULT_ALL_DOMAIN_STAGE_LIST_PATH),
    )?;
    let stage_tool_table = render_all_domain_stage_tool_table(
        repo_root,
        PathBuf::from(DEFAULT_ALL_DOMAIN_STAGE_TOOL_TABLE_PATH),
    )?;
    let expected_results = render_all_domain_expected_benchmark_results(
        repo_root,
        PathBuf::from(DEFAULT_ALL_DOMAIN_EXPECTED_BENCHMARK_RESULTS_PATH),
    )?;
    let rendered_commands = render_all_domain_commands(
        repo_root,
        PathBuf::from(DEFAULT_ALL_DOMAIN_RENDERED_COMMANDS_PATH),
    )?;
    let parser_collector = render_all_domain_parser_collector(
        repo_root,
        PathBuf::from(DEFAULT_ALL_DOMAIN_PARSER_COLLECTOR_PATH),
    )?;
    let acquired_dependency;
    let full_report = if let Some(report) = dependency {
        report
    } else {
        acquired_dependency = render_full_benchmark_report(
            repo_root,
            PathBuf::from(DEFAULT_FULL_BENCHMARK_REPORT_MARKDOWN_PATH),
        )?;
        &acquired_dependency
    };
    let real_smoke = render_real_smoke_core_subset(
        repo_root,
        PathBuf::from(DEFAULT_REAL_SMOKE_CORE_SUBSET_PATH),
    )?;

    let report = evaluate_full_benchmark_dashboard(
        FullBenchmarkDashboardEvidence {
            stage_inventory: &stage_inventory,
            stage_tool_table: &stage_tool_table,
            expected_results: &expected_results,
            rendered_commands: &rendered_commands,
            parser_collector: &parser_collector,
            full_report,
            real_smoke: &real_smoke,
        },
        FullBenchmarkDashboardPaths {
            markdown: DEFAULT_FULL_BENCHMARK_DASHBOARD_MARKDOWN_PATH,
            json: DEFAULT_FULL_BENCHMARK_DASHBOARD_JSON_PATH,
            stage_inventory: DEFAULT_ALL_DOMAIN_STAGE_LIST_PATH,
        },
        &path_relative_to_repo(repo_root, &markdown_output_path),
        &path_relative_to_repo(repo_root, &json_output_path),
    )?;

    if let Some(parent) = markdown_output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&markdown_output_path, render_full_benchmark_dashboard_markdown(&report))
        .with_context(|| format!("write {}", markdown_output_path.display()))?;
    bijux_dna_infra::atomic_write_json(&json_output_path, &report)?;
    Ok(report)
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

fn derive_json_output_path(markdown_output_path: &Path) -> PathBuf {
    markdown_output_path.with_extension("json")
}
