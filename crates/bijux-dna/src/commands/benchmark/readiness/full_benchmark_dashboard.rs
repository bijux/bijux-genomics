use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
pub(crate) use bijux_dna_api::v1::api::bench::FullBenchmarkDashboardReport;
use bijux_dna_api::v1::api::bench::{
    evaluate_full_benchmark_dashboard, render_full_benchmark_dashboard_markdown,
    FullBenchmarkDashboardEvidence, FullBenchmarkDashboardPaths,
};

use super::all_domain_expected_benchmark_results::{
    render_all_domain_expected_benchmark_results, AllDomainExpectedBenchmarkResultsReport,
    DEFAULT_ALL_DOMAIN_EXPECTED_BENCHMARK_RESULTS_PATH,
};
use super::all_domain_parser_collector::{
    render_all_domain_parser_collector, AllDomainParserCollectorReport,
    DEFAULT_ALL_DOMAIN_PARSER_COLLECTOR_PATH,
};
use super::all_domain_rendered_commands::{
    render_all_domain_commands, AllDomainRenderedCommandsReport,
    DEFAULT_ALL_DOMAIN_RENDERED_COMMANDS_PATH,
};
use super::all_domain_stage_tool_table::{
    render_all_domain_stage_tool_table, AllDomainStageToolTableReport,
    DEFAULT_ALL_DOMAIN_STAGE_TOOL_TABLE_PATH,
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
    render_full_benchmark_dashboard_with_dependencies(
        repo_root,
        markdown_output_path,
        DashboardReportDependencies::default(),
    )
}

/// Successful observations from one request whose inputs and referenced artifacts remain unchanged.
#[derive(Default)]
pub(crate) struct DashboardReportDependencies<'a> {
    pub(crate) stage_tools: Option<&'a AllDomainStageToolTableReport>,
    pub(crate) expected_results: Option<&'a AllDomainExpectedBenchmarkResultsReport>,
    pub(crate) commands: Option<&'a AllDomainRenderedCommandsReport>,
    pub(crate) parser: Option<&'a AllDomainParserCollectorReport>,
    pub(crate) full_report: Option<&'a FullBenchmarkReport>,
}

/// Borrow successful request evidence; absent reports retain their ordinary acquisition and errors.
pub(crate) fn render_full_benchmark_dashboard_with_dependencies(
    repo_root: &Path,
    markdown_output_path: PathBuf,
    dependencies: DashboardReportDependencies<'_>,
) -> Result<FullBenchmarkDashboardReport> {
    let markdown_output_path = repo_relative_path(repo_root, &markdown_output_path);
    let json_output_path = derive_json_output_path(&markdown_output_path);

    let stage_inventory = render_all_domain_stage_inventory(
        repo_root,
        &[BenchLocalDomain::Fastq, BenchLocalDomain::Bam, BenchLocalDomain::Vcf],
        PathBuf::from(DEFAULT_ALL_DOMAIN_STAGE_LIST_PATH),
    )?;
    let acquired_stage_tool_table;
    let stage_tool_table = if let Some(report) = dependencies.stage_tools {
        report
    } else {
        acquired_stage_tool_table = render_all_domain_stage_tool_table(
            repo_root,
            PathBuf::from(DEFAULT_ALL_DOMAIN_STAGE_TOOL_TABLE_PATH),
        )?;
        &acquired_stage_tool_table
    };
    let acquired_expected_results;
    let expected_results = if let Some(report) = dependencies.expected_results {
        report
    } else {
        acquired_expected_results = render_all_domain_expected_benchmark_results(
            repo_root,
            PathBuf::from(DEFAULT_ALL_DOMAIN_EXPECTED_BENCHMARK_RESULTS_PATH),
        )?;
        &acquired_expected_results
    };
    let acquired_rendered_commands;
    let rendered_commands = if let Some(report) = dependencies.commands {
        report
    } else {
        acquired_rendered_commands = render_all_domain_commands(
            repo_root,
            PathBuf::from(DEFAULT_ALL_DOMAIN_RENDERED_COMMANDS_PATH),
        )?;
        &acquired_rendered_commands
    };
    let acquired_parser_collector;
    let parser_collector = if let Some(report) = dependencies.parser {
        report
    } else {
        acquired_parser_collector = render_all_domain_parser_collector(
            repo_root,
            PathBuf::from(DEFAULT_ALL_DOMAIN_PARSER_COLLECTOR_PATH),
        )?;
        &acquired_parser_collector
    };
    let acquired_dependency;
    let full_report = if let Some(report) = dependencies.full_report {
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
            stage_tool_table,
            expected_results,
            rendered_commands,
            parser_collector,
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use bijux_dna_api::v1::api::bench::{
        AllDomainExpectedBenchmarkResultRow, AllDomainStageToolTableRow,
        ALL_DOMAIN_EXPECTED_BENCHMARK_RESULTS_SCHEMA_V1,
        ALL_DOMAIN_STAGE_TOOL_TABLE_SCHEMA_VERSION,
    };

    use super::*;

    fn stage_tools_report() -> AllDomainStageToolTableReport {
        AllDomainStageToolTableReport {
            schema_version: ALL_DOMAIN_STAGE_TOOL_TABLE_SCHEMA_VERSION.to_string(),
            output_path: DEFAULT_ALL_DOMAIN_STAGE_TOOL_TABLE_PATH.to_string(),
            row_count: 1,
            benchmark_ready_row_count: 1,
            benchmark_ready_unique_binding_count: 1,
            domain_counts: BTreeMap::from([("bam".to_string(), 1)]),
            benchmark_ready_domain_counts: BTreeMap::from([("bam".to_string(), 1)]),
            rows: vec![AllDomainStageToolTableRow {
                domain: "bam".to_string(),
                stage_id: "bam.validate".to_string(),
                tool_id: "declared-tool".to_string(),
                corpus_id: "truth".to_string(),
                asset_profile_id: "diploid".to_string(),
                adapter_id: "adapter".to_string(),
                parser_id: "parser".to_string(),
                benchmark_status: "benchmark_ready".to_string(),
            }],
        }
    }

    fn expected_results_report() -> AllDomainExpectedBenchmarkResultsReport {
        AllDomainExpectedBenchmarkResultsReport {
            schema_version: ALL_DOMAIN_EXPECTED_BENCHMARK_RESULTS_SCHEMA_V1.to_string(),
            output_path: DEFAULT_ALL_DOMAIN_EXPECTED_BENCHMARK_RESULTS_PATH.to_string(),
            row_count: 1,
            result_id_count: 1,
            stage_count: 1,
            tool_count: 1,
            corpus_count: 1,
            asset_profile_count: 1,
            domain_counts: BTreeMap::from([("bam".to_string(), 1)]),
            report_section_counts: BTreeMap::from([("validation".to_string(), 1)]),
            rows: vec![AllDomainExpectedBenchmarkResultRow {
                result_id: "result".to_string(),
                domain: "bam".to_string(),
                stage_id: "bam.validate".to_string(),
                tool_id: "declared-tool".to_string(),
                corpus_id: "truth".to_string(),
                asset_profile_id: "diploid".to_string(),
                expected_outputs: vec!["report".to_string()],
                expected_metrics: vec![],
                report_section: "validation".to_string(),
            }],
        }
    }

    #[test]
    fn partial_dashboard_dependencies_preserve_real_acquisition_failures() -> Result<()> {
        let repository = crate::commands::support::workspace_root::resolve_repo_root()?;
        let scratch = repository.join("artifacts/tests/dashboard-acquisition");
        fs::create_dir_all(&scratch)?;
        let fixture =
            tempfile::Builder::new().prefix("dashboard-acquisition-").tempdir_in(scratch)?;
        let root = fixture.path();
        fs::create_dir_all(root.join("configs/bench/local"))?;
        for domain in ["fastq", "bam", "vcf"] {
            let path = PathBuf::from(format!("configs/bench/local/{domain}-stage-matrix.toml"));
            fs::copy(repository.join(&path), root.join(path))?;
        }
        let stage_tools = stage_tools_report();
        let expected_results = expected_results_report();
        let dashboard_path = PathBuf::from(DEFAULT_FULL_BENCHMARK_DASHBOARD_MARKDOWN_PATH);
        let error = render_full_benchmark_dashboard_with_dependencies(
            root,
            dashboard_path.clone(),
            DashboardReportDependencies {
                stage_tools: Some(&stage_tools),
                expected_results: Some(&expected_results),
                ..Default::default()
            },
        )
        .err()
        .context("missing commands must invoke the real command renderer")?;
        let command_error = render_all_domain_commands(
            root,
            PathBuf::from(DEFAULT_ALL_DOMAIN_RENDERED_COMMANDS_PATH),
        )
        .err()
        .context("fixture must lack the governed execution-mode input")?;
        assert_eq!(format!("{error:#}"), format!("{command_error:#}"));
        assert!(error.to_string().contains(
            "validate governed benchmark tool execution modes before all-domain rendering"
        ));
        assert!(root.join(DEFAULT_ALL_DOMAIN_STAGE_LIST_PATH).is_file());

        let error = render_full_benchmark_dashboard_with_dependencies(
            root,
            dashboard_path.clone(),
            DashboardReportDependencies {
                expected_results: Some(&expected_results),
                ..Default::default()
            },
        )
        .err()
        .context("missing stage-tool evidence must retain its earlier renderer failure")?;
        let stage_tool_error = render_all_domain_stage_tool_table(
            root,
            PathBuf::from(DEFAULT_ALL_DOMAIN_STAGE_TOOL_TABLE_PATH),
        )
        .err()
        .context("fixture must lack the governed stage-tool assets input")?;
        assert_eq!(format!("{error:#}"), format!("{stage_tool_error:#}"));
        assert!(error.to_string().contains("stage-tool-assets.toml"));
        for path in [
            PathBuf::from(DEFAULT_ALL_DOMAIN_STAGE_TOOL_TABLE_PATH),
            PathBuf::from(DEFAULT_ALL_DOMAIN_EXPECTED_BENCHMARK_RESULTS_PATH),
            dashboard_path.with_extension("json"),
            dashboard_path,
        ] {
            assert!(!root.join(path).exists());
        }
        Ok(())
    }
}
