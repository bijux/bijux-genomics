use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Serialize;

use super::all_domain_expected_benchmark_results::{
    render_all_domain_expected_benchmark_results, AllDomainExpectedBenchmarkResultsReport,
    DEFAULT_ALL_DOMAIN_EXPECTED_BENCHMARK_RESULTS_PATH,
};
use super::all_domain_harness_ready::{
    render_all_domain_harness_ready, DEFAULT_ALL_DOMAIN_HARNESS_READY_PATH,
};
use super::all_domain_output_declarations::{
    render_all_domain_output_declarations, AllDomainOutputDeclarationStatus,
    DEFAULT_ALL_DOMAIN_OUTPUT_DECLARATIONS_PATH,
};
use super::all_domain_parser_collector::{
    render_all_domain_parser_collector, AllDomainParserCollectorSourceKind,
    DEFAULT_ALL_DOMAIN_PARSER_COLLECTOR_PATH,
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
    render_full_benchmark_dashboard, DEFAULT_FULL_BENCHMARK_DASHBOARD_MARKDOWN_PATH,
};
use super::full_benchmark_report::{
    render_full_benchmark_report, DEFAULT_FULL_BENCHMARK_REPORT_MARKDOWN_PATH,
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
    render_all_domain_slurm_submit_manifest, DEFAULT_ALL_DOMAIN_SLURM_SUBMIT_MANIFEST_PATH,
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
    "benchmarks/readiness/FASTQ_BAM_VCF_OPERATIONAL_BENCHMARK_READY.json";
const OPERATIONAL_BENCHMARK_READY_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.operational_benchmark_ready.v1";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct BindingKey {
    domain: String,
    stage_id: String,
    tool_id: String,
    corpus_id: String,
    asset_profile_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct PairKey {
    domain: String,
    stage_id: String,
    tool_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct OperationalBenchmarkReadyCheck {
    pub(crate) surface_id: String,
    pub(crate) output_path: String,
    pub(crate) ok: bool,
    pub(crate) detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub(crate) struct OperationalBenchmarkReadyBlocker {
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) corpus_id: String,
    pub(crate) asset_profile_id: String,
    pub(crate) blocker_type: String,
    pub(crate) blocker_path: String,
    pub(crate) detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct OperationalBenchmarkReadyReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) checked_surface_count: usize,
    pub(crate) passed_surface_count: usize,
    pub(crate) failed_surface_count: usize,
    pub(crate) benchmark_ready_row_count: usize,
    pub(crate) blocker_count: usize,
    pub(crate) missing_result_row_count: usize,
    pub(crate) insufficient_data_row_count: usize,
    pub(crate) unsupported_pair_row_count: usize,
    pub(crate) ok: bool,
    pub(crate) checks: Vec<OperationalBenchmarkReadyCheck>,
    pub(crate) blockers: Vec<OperationalBenchmarkReadyBlocker>,
}

pub(crate) fn run_render_operational_benchmark_ready(
    args: &parse::BenchReadinessRenderOperationalBenchmarkReadyArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
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
                "parser_fixture_row_count={}, benchmark_ready_parser_row_count={}",
                report.parser_fixture_row_count, report.benchmark_ready_parser_row_count
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

    let all_domain_harness_ready = render_gate_check(
        &mut checks,
        "all_domain_harness_ready",
        DEFAULT_ALL_DOMAIN_HARNESS_READY_PATH,
        || {
            render_all_domain_harness_ready(
                repo_root,
                PathBuf::from(DEFAULT_ALL_DOMAIN_HARNESS_READY_PATH),
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
    if all_domain_harness_ready.is_none() {
        blockers.insert(global_blocker(
            "cross",
            "benchmark.harness",
            "all_domain",
            "benchmark_ready",
            "benchmark_ready",
            "surface_render_failed",
            DEFAULT_ALL_DOMAIN_HARNESS_READY_PATH,
            "All-domain harness readiness failed",
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
            render_all_domain_slurm_scripts(
                repo_root,
                PathBuf::from(DEFAULT_ALL_DOMAIN_SLURM_DRY_RUN_ROOT),
            )?;
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
            render_all_domain_slurm_scripts(
                repo_root,
                PathBuf::from(DEFAULT_ALL_DOMAIN_SLURM_DRY_RUN_ROOT),
            )?;
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
            render_all_domain_slurm_submit_manifest(
                repo_root,
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
        "runs/bench/slurm-dry-run/all-domains/path-convention-check.json",
        || {
            validate_all_domain_slurm_result_paths(
                repo_root,
                PathBuf::from(DEFAULT_ALL_DOMAIN_SLURM_DRY_RUN_ROOT),
                PathBuf::from(DEFAULT_ALL_DOMAIN_SLURM_SUBMIT_MANIFEST_PATH),
                PathBuf::from("runs/bench/slurm-dry-run/all-domains/path-convention-check.json"),
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
            "runs/bench/slurm-dry-run/all-domains/path-convention-check.json",
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
            render_full_benchmark_report(
                repo_root,
                PathBuf::from(DEFAULT_FULL_BENCHMARK_REPORT_MARKDOWN_PATH),
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
            render_full_benchmark_dashboard(
                repo_root,
                PathBuf::from(DEFAULT_FULL_BENCHMARK_DASHBOARD_MARKDOWN_PATH),
            )
        },
        |report| {
            format!(
                "total_expected_jobs={}, ready_jobs={}, blocked_jobs={}",
                report.total_expected_jobs, report.ready_jobs, report.blocked_jobs
            )
        },
    );

    if let (Some(expected), Some(table)) = (&expected_results, &stage_tool_table) {
        let expected_keys =
            expected.rows.iter().map(binding_key_from_expected).collect::<BTreeSet<_>>();
        for row in table.rows.iter().filter(|row| row.benchmark_status == "benchmark_ready") {
            let binding = binding_key_from_table(row);
            if !expected_keys.contains(&binding) {
                blockers.insert(binding_blocker(
                    &binding,
                    "expected_result_missing",
                    &expected.output_path,
                    "benchmark-ready binding is missing a canonical expected-result row",
                ));
            }
        }
    }

    if let (Some(expected), Some(rendered)) = (&expected_results, &rendered_commands) {
        let rendered_result_ids =
            rendered.rows.iter().map(|row| row.result_id.as_str()).collect::<BTreeSet<_>>();
        for row in &expected.rows {
            if !rendered_result_ids.contains(row.result_id.as_str()) {
                blockers.insert(binding_blocker(
                    &binding_key_from_expected(row),
                    "adapter_command_missing",
                    &rendered.output_path,
                    "canonical benchmark result is missing rendered command coverage",
                ));
            }
        }
    }

    if let (Some(expected), Some(outputs)) = (&expected_results, &output_declarations) {
        let output_by_result = outputs
            .rows
            .iter()
            .map(|row| (row.result_id.as_str(), row))
            .collect::<BTreeMap<_, _>>();
        for row in &expected.rows {
            match output_by_result.get(row.result_id.as_str()) {
                None => {
                    blockers.insert(binding_blocker(
                        &binding_key_from_expected(row),
                        "output_declaration_missing",
                        &outputs.output_path,
                        "canonical benchmark result is missing output declarations",
                    ));
                }
                Some(declaration)
                    if declaration.status != AllDomainOutputDeclarationStatus::Complete =>
                {
                    blockers.insert(binding_blocker(
                        &binding_key_from_expected(row),
                        "output_declaration_incomplete",
                        &outputs.output_path,
                        "canonical benchmark result has incomplete declared outputs",
                    ));
                }
                Some(_) => {}
            }
        }
    }

    if let (Some(expected), Some(parser_rows)) = (&expected_results, &parser_collector) {
        let fake_run_result_ids = parser_rows
            .rows
            .iter()
            .filter(|row| row.source_kind == AllDomainParserCollectorSourceKind::FakeRun)
            .filter_map(|row| row.result_id.as_deref())
            .collect::<BTreeSet<_>>();
        for row in &expected.rows {
            if !fake_run_result_ids.contains(row.result_id.as_str()) {
                blockers.insert(binding_blocker(
                    &binding_key_from_expected(row),
                    "parser_evidence_missing",
                    &parser_rows.output_path,
                    "canonical benchmark result is missing fake-run parser evidence",
                ));
            }
        }
    }

    if let (Some(expected), Some(resources)) = (&expected_results, &resource_report) {
        let resource_pairs =
            resources.rows.iter().map(binding_key_from_resource).collect::<BTreeSet<_>>();
        for row in &expected.rows {
            let binding = binding_key_from_expected(row);
            if !resource_pairs.contains(&pair_key_from_expected(row)) {
                blockers.insert(binding_blocker(
                    &binding,
                    "resource_hint_missing",
                    &resources.config_path,
                    "canonical benchmark result is missing governed resource hints",
                ));
            }
        }
    }

    if let (Some(expected), Some(table)) = (&expected_results, &stage_tool_table) {
        let table_by_binding = table
            .rows
            .iter()
            .filter(|row| row.benchmark_status == "benchmark_ready")
            .map(|row| (binding_key_from_table(row), row))
            .collect::<BTreeMap<_, _>>();
        for row in &expected.rows {
            let binding = binding_key_from_expected(row);
            match table_by_binding.get(&binding) {
                None => {
                    blockers.insert(binding_blocker(
                    &binding,
                    "stage_tool_binding_missing",
                    &table.output_path,
                    "canonical benchmark result is missing from the all-domain stage/tool table",
                ));
                }
                Some(table_row)
                    if table_row.corpus_id.trim().is_empty()
                        || table_row.asset_profile_id.trim().is_empty()
                        || table_row.adapter_id.trim().is_empty()
                        || table_row.parser_id.trim().is_empty() =>
                {
                    blockers.insert(binding_blocker(
                        &binding,
                        "binding_metadata_incomplete",
                        &table.output_path,
                        "stage/tool table binding is missing corpus, asset, adapter, or parser identity",
                    ));
                }
                Some(_) => {}
            };
        }
    }

    checks.push(fastq_bam_binding_check(
        &expected_results,
        stage_tool_table.is_some(),
        rendered_commands.is_some(),
        output_declarations.is_some(),
        parser_collector.is_some(),
        resource_report.is_some(),
        &blockers,
    ));

    if let Some(report) = &result_collector {
        if report.missing_result_status_count == 0 {
            blockers.insert(global_blocker(
                "cross",
                "benchmark.result_collector",
                "collector",
                "benchmark_ready",
                "benchmark_ready",
                "missing_result_omitted",
                &report.output_path,
                "full benchmark result collector omitted missing_result rows",
            ));
        }
        if report.insufficient_data_status_count == 0 {
            blockers.insert(global_blocker(
                "vcf",
                "vcf.demography",
                "ibdne",
                "vcf_production_regression",
                "json_ibd_segments",
                "insufficient_data_omitted",
                &report.output_path,
                "full benchmark result collector omitted insufficient_data rows",
            ));
        }
        if report.unsupported_pair_status_count == 0 {
            blockers.insert(global_blocker(
                "vcf",
                "vcf.filter",
                "samtools",
                "vcf_production_regression",
                "vcf_cohort",
                "unsupported_pair_omitted",
                &report.output_path,
                "full benchmark result collector omitted unsupported_pair rows",
            ));
        }
    } else {
        blockers.insert(global_blocker(
            "cross",
            "benchmark.result_collector",
            "collector",
            "benchmark_ready",
            "benchmark_ready",
            "surface_render_failed",
            DEFAULT_FULL_BENCHMARK_RESULT_COLLECTOR_PATH,
            "Full benchmark result collector failed",
        ));
    }

    if let Some(report) = &full_report {
        if !report.passes_behavior_test {
            blockers.insert(global_blocker(
                "cross",
                "benchmark.report",
                "report",
                "benchmark_ready",
                "benchmark_ready",
                "report_contract_failed",
                &report.json_output_path,
                "full benchmark report failed its governed behavior contract",
            ));
        }
        if report.missing_result_row_count == 0 || report.unsupported_pair_row_count == 0 {
            blockers.insert(global_blocker(
                "cross",
                "benchmark.report",
                "report",
                "benchmark_ready",
                "benchmark_ready",
                "report_rows_omitted",
                &report.json_output_path,
                "full benchmark report omitted missing-result or unsupported rows",
            ));
        }
    } else {
        blockers.insert(global_blocker(
            "cross",
            "benchmark.report",
            "report",
            "benchmark_ready",
            "benchmark_ready",
            "surface_render_failed",
            DEFAULT_FULL_BENCHMARK_REPORT_MARKDOWN_PATH,
            "Full benchmark report failed",
        ));
    }

    if let Some(report) = &dashboard {
        if !report.passes_behavior_test {
            blockers.insert(global_blocker(
                "cross",
                "benchmark.dashboard",
                "dashboard",
                "benchmark_ready",
                "benchmark_ready",
                "dashboard_contract_failed",
                &report.json_output_path,
                "full benchmark dashboard failed its governed behavior contract",
            ));
        }
    } else {
        blockers.insert(global_blocker(
            "cross",
            "benchmark.dashboard",
            "dashboard",
            "benchmark_ready",
            "benchmark_ready",
            "surface_render_failed",
            DEFAULT_FULL_BENCHMARK_DASHBOARD_MARKDOWN_PATH,
            "Full benchmark dashboard failed",
        ));
    }

    if vcf_catalog_ready.is_none() || vcf_smoke_ready.is_none() {
        blockers.insert(global_blocker(
            "vcf",
            "readiness.local_validation",
            "governed_validation",
            "vcf_mini",
            "vcf_cohort",
            "vcf_stage_untested",
            DEFAULT_VCF_SMOKE_SUITE_READY_PATH,
            "VCF catalog or local smoke readiness is not green",
        ));
    }

    let checks = checks;
    let blocker_count = blockers.len();
    let passed_surface_count = checks.iter().filter(|check| check.ok).count();
    let failed_surface_count = checks.len().saturating_sub(passed_surface_count);
    let benchmark_ready_row_count = expected_results.as_ref().map_or(0, |report| report.row_count);
    let missing_result_row_count =
        result_collector.as_ref().map_or(0, |report| report.missing_result_status_count);
    let insufficient_data_row_count =
        result_collector.as_ref().map_or(0, |report| report.insufficient_data_status_count);
    let unsupported_pair_row_count =
        result_collector.as_ref().map_or(0, |report| report.unsupported_pair_status_count);
    let ok = blocker_count == 0 && failed_surface_count == 0;

    let report = OperationalBenchmarkReadyReport {
        schema_version: OPERATIONAL_BENCHMARK_READY_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &absolute_output_path),
        checked_surface_count: checks.len(),
        passed_surface_count,
        failed_surface_count,
        benchmark_ready_row_count,
        blocker_count,
        missing_result_row_count,
        insufficient_data_row_count,
        unsupported_pair_row_count,
        ok,
        checks,
        blockers: blockers.into_iter().collect(),
    };
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

fn fastq_bam_binding_check(
    expected_results: &Option<AllDomainExpectedBenchmarkResultsReport>,
    has_stage_tool_table: bool,
    has_rendered_commands: bool,
    has_output_declarations: bool,
    has_parser_collector: bool,
    has_resource_report: bool,
    blockers: &BTreeSet<OperationalBenchmarkReadyBlocker>,
) -> OperationalBenchmarkReadyCheck {
    match expected_results {
        Some(report) => {
            let binding_count = report
                .rows
                .iter()
                .filter(|row| row.domain == "fastq" || row.domain == "bam")
                .count();
            let blocker_count = blockers
                .iter()
                .filter(|blocker| blocker.domain == "fastq" || blocker.domain == "bam")
                .count();
            let mut missing_prerequisites = Vec::new();
            if !has_stage_tool_table {
                missing_prerequisites.push("all_domain_stage_tool_table");
            }
            if !has_rendered_commands {
                missing_prerequisites.push("all_domain_rendered_commands");
            }
            if !has_output_declarations {
                missing_prerequisites.push("all_domain_output_declarations");
            }
            if !has_parser_collector {
                missing_prerequisites.push("all_domain_parser_collector");
            }
            if !has_resource_report {
                missing_prerequisites.push("stage_tool_resources");
            }
            let ok = binding_count > 0 && blocker_count == 0 && missing_prerequisites.is_empty();
            let detail = if ok {
                format!(
                    "validated {} FASTQ/BAM benchmark-ready bindings through expected-result, command, parser, resource, and output coverage",
                    binding_count
                )
            } else if !missing_prerequisites.is_empty() {
                format!(
                    "FASTQ/BAM benchmark binding coverage is missing prerequisite surfaces: {}",
                    missing_prerequisites.join(", ")
                )
            } else {
                format!(
                    "FASTQ/BAM benchmark binding coverage found {} blockers across {} bindings",
                    blocker_count, binding_count
                )
            };
            OperationalBenchmarkReadyCheck {
                surface_id: "fastq_bam_benchmark_binding_coverage".to_string(),
                output_path: report.output_path.clone(),
                ok,
                detail,
            }
        }
        None => failed_check(
            "fastq_bam_benchmark_binding_coverage",
            DEFAULT_ALL_DOMAIN_EXPECTED_BENCHMARK_RESULTS_PATH,
            "all-domain expected benchmark results did not render".to_string(),
        ),
    }
}

fn binding_key_from_expected(
    row: &super::all_domain_expected_benchmark_results::AllDomainExpectedBenchmarkResultRow,
) -> BindingKey {
    BindingKey {
        domain: row.domain.clone(),
        stage_id: row.stage_id.clone(),
        tool_id: row.tool_id.clone(),
        corpus_id: row.corpus_id.clone(),
        asset_profile_id: row.asset_profile_id.clone(),
    }
}

fn binding_key_from_table(
    row: &super::all_domain_stage_tool_table::AllDomainStageToolTableRow,
) -> BindingKey {
    BindingKey {
        domain: row.domain.clone(),
        stage_id: row.stage_id.clone(),
        tool_id: row.tool_id.clone(),
        corpus_id: row.corpus_id.clone(),
        asset_profile_id: row.asset_profile_id.clone(),
    }
}

fn binding_key_from_resource(row: &super::stage_tool_resources::StageToolResourceRow) -> PairKey {
    PairKey {
        domain: row.domain.clone(),
        stage_id: row.stage_id.clone(),
        tool_id: row.tool_id.clone(),
    }
}

fn pair_key_from_expected(
    row: &super::all_domain_expected_benchmark_results::AllDomainExpectedBenchmarkResultRow,
) -> PairKey {
    PairKey {
        domain: row.domain.clone(),
        stage_id: row.stage_id.clone(),
        tool_id: row.tool_id.clone(),
    }
}

fn benchmark_asset_profile_id_for_resource_row(
    row: &super::stage_tool_resources::StageToolResourceRow,
) -> String {
    match row.domain.as_str() {
        "fastq" => "benchmark_ready".to_string(),
        "bam" => "benchmark_ready".to_string(),
        "vcf" => match row.stage_id.as_str() {
            "vcf.call" | "vcf.filter" | "vcf.stats" | "vcf.qc" => "vcf_cohort".to_string(),
            "vcf.prepare_reference_panel" => "vcf_reference_panel".to_string(),
            "vcf.phasing" | "vcf.impute" | "vcf.imputation_metrics" => {
                "vcf_cohort_with_panel".to_string()
            }
            _ => "vcf_cohort".to_string(),
        },
        _ => "benchmark_ready".to_string(),
    }
}

fn binding_blocker(
    binding: &BindingKey,
    blocker_type: &str,
    blocker_path: &str,
    detail: &str,
) -> OperationalBenchmarkReadyBlocker {
    OperationalBenchmarkReadyBlocker {
        domain: binding.domain.clone(),
        stage_id: binding.stage_id.clone(),
        tool_id: binding.tool_id.clone(),
        corpus_id: binding.corpus_id.clone(),
        asset_profile_id: binding.asset_profile_id.clone(),
        blocker_type: blocker_type.to_string(),
        blocker_path: blocker_path.to_string(),
        detail: detail.to_string(),
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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{benchmark_asset_profile_id_for_resource_row, binding_blocker, BindingKey};
    #[cfg(feature = "bam_downstream")]
    use super::{render_operational_benchmark_ready, DEFAULT_OPERATIONAL_BENCHMARK_READY_PATH};
    use crate::commands::benchmark::readiness::stage_tool_resources::StageToolResourceRow;

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[cfg(feature = "bam_downstream")]
    #[test]
    fn operational_benchmark_ready_reports_green_governed_surface() {
        let root = repo_root();
        let report = render_operational_benchmark_ready(
            &root,
            PathBuf::from(DEFAULT_OPERATIONAL_BENCHMARK_READY_PATH),
        )
        .expect("render operational benchmark readiness");

        assert_eq!(report.schema_version, "bijux.bench.readiness.operational_benchmark_ready.v1");
        assert_eq!(
            report.output_path,
            "benchmarks/readiness/FASTQ_BAM_VCF_OPERATIONAL_BENCHMARK_READY.json"
        );
        assert_eq!(report.benchmark_ready_row_count, 130);
        assert_eq!(report.blocker_count, 0);
        assert_eq!(report.missing_result_row_count, 3);
        assert_eq!(report.insufficient_data_row_count, 1);
        assert_eq!(report.unsupported_pair_row_count, 1);
        assert!(report.ok);
        assert!(report.checks.iter().all(|check| check.ok));
    }

    #[test]
    fn benchmark_resource_asset_profile_maps_vcf_panel_workflow_rows() {
        let row = StageToolResourceRow {
            domain: "vcf".to_string(),
            stage_id: "vcf.impute".to_string(),
            tool_id: "beagle".to_string(),
            threads: 8,
            memory_gb: 16,
            walltime_minutes: 60,
            scratch_gb: 8,
            resource_origin: "planner_stage_constraints_with_stage_walltime_profile".to_string(),
        };

        assert_eq!(benchmark_asset_profile_id_for_resource_row(&row), "vcf_cohort_with_panel");
    }

    #[test]
    fn binding_blocker_keeps_exact_blocker_tuple_fields() {
        let binding = BindingKey {
            domain: "vcf".to_string(),
            stage_id: "vcf.stats".to_string(),
            tool_id: "bcftools".to_string(),
            corpus_id: "vcf_production_regression".to_string(),
            asset_profile_id: "vcf_cohort".to_string(),
        };

        let blocker = binding_blocker(
            &binding,
            "resource_hint_missing",
            "benchmarks/configs/local/stage-tool-resources.toml",
            "canonical benchmark result is missing governed resource hints",
        );

        assert_eq!(blocker.domain, "vcf");
        assert_eq!(blocker.stage_id, "vcf.stats");
        assert_eq!(blocker.tool_id, "bcftools");
        assert_eq!(blocker.corpus_id, "vcf_production_regression");
        assert_eq!(blocker.asset_profile_id, "vcf_cohort");
        assert_eq!(blocker.blocker_type, "resource_hint_missing");
        assert_eq!(blocker.blocker_path, "benchmarks/configs/local/stage-tool-resources.toml");
    }
}
