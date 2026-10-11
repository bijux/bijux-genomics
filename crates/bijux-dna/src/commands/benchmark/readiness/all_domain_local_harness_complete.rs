use bijux_dna_api::v1::api::bench::harness::{
    validate_all_domain_harness_alignment, HarnessAlignmentEvidence,
};
use std::collections::BTreeSet;
use std::env;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde::Serialize;

use super::all_domain_completion_check::DEFAULT_ALL_DOMAIN_COMPLETION_CHECK_PATH;
use super::all_domain_expected_benchmark_results::{
    render_all_domain_expected_benchmark_results, AllDomainExpectedBenchmarkResultsReport,
    DEFAULT_ALL_DOMAIN_EXPECTED_BENCHMARK_RESULTS_PATH,
};
use super::all_domain_failure_classification::{
    render_all_domain_failure_classification, DEFAULT_ALL_DOMAIN_FAILURE_CLASSIFICATION_PATH,
};
use super::all_domain_missing_result_audit::DEFAULT_ALL_DOMAIN_MISSING_RESULT_TEST_PATH;
use super::all_domain_output_declarations::{
    render_all_domain_output_declarations, AllDomainOutputDeclarationsReport,
    DEFAULT_ALL_DOMAIN_OUTPUT_DECLARATIONS_PATH,
};
use super::all_domain_parser_collector::{
    render_all_domain_parser_collector, AllDomainParserCollectorReport,
    AllDomainParserCollectorSourceKind, DEFAULT_ALL_DOMAIN_PARSER_COLLECTOR_PATH,
};
use super::all_domain_rendered_commands::{
    render_all_domain_commands, AllDomainRenderedCommandsReport,
    DEFAULT_ALL_DOMAIN_RENDERED_COMMANDS_PATH,
};
use super::all_domain_stage_tool_table::{
    render_all_domain_stage_tool_table, AllDomainStageToolTableReport,
    DEFAULT_ALL_DOMAIN_STAGE_TOOL_TABLE_PATH,
};
use crate::commands::benchmark::local_all_domain_fake_failures::DEFAULT_ALL_DOMAIN_FAKE_FAILURE_ROOT;
use crate::commands::benchmark::local_all_domain_fake_runs::DEFAULT_ALL_DOMAIN_FAKE_RUN_ROOT;
use crate::commands::benchmark::local_real_smoke_core_subset::{
    render_real_smoke_core_subset, DEFAULT_REAL_SMOKE_CORE_SUBSET_PATH,
};
use crate::commands::benchmark::local_stage_inventory::{
    render_all_domain_stage_inventory, BenchLocalDomain, DEFAULT_ALL_DOMAIN_STAGE_LIST_PATH,
};
use crate::commands::benchmark::local_stage_result_manifest::path_relative_to_repo;
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_ALL_DOMAIN_LOCAL_HARNESS_COMPLETE_PATH: &str =
    "artifacts/benchmark-smoke/readiness/all-domains/ALL_DOMAIN_LOCAL_HARNESS_COMPLETE.json";
const ALL_DOMAIN_LOCAL_HARNESS_COMPLETE_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.all_domain_local_harness_complete.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct AllDomainLocalHarnessCompleteGoalCheck {
    pub(crate) goal_id: u32,
    pub(crate) surface: String,
    pub(crate) output_path: Option<String>,
    pub(crate) ok: bool,
    pub(crate) detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AllDomainLocalHarnessCompleteReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) checked_goal_count: usize,
    pub(crate) passed_goal_count: usize,
    pub(crate) failed_goal_count: usize,
    pub(crate) failing_goal_ids: Vec<u32>,
    pub(crate) all_domain_stage_count: usize,
    pub(crate) benchmark_ready_binding_count: usize,
    pub(crate) expected_result_row_count: usize,
    pub(crate) rendered_command_row_count: usize,
    pub(crate) output_declaration_row_count: usize,
    pub(crate) fake_run_result_count: usize,
    pub(crate) fake_run_output_count: usize,
    pub(crate) fake_failure_result_count: usize,
    pub(crate) fake_failure_output_count: usize,
    pub(crate) completion_row_count: usize,
    pub(crate) parser_collector_row_count: usize,
    pub(crate) missing_result_row_count: usize,
    pub(crate) failure_class_count: usize,
    pub(crate) real_smoke_execution_count: usize,
    pub(crate) ok: bool,
    pub(crate) checks: Vec<AllDomainLocalHarnessCompleteGoalCheck>,
}

pub(crate) fn run_render_all_domain_local_harness_complete(
    args: &parse::BenchReadinessRenderAllDomainLocalHarnessCompleteArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_all_domain_local_harness_complete(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_ALL_DOMAIN_LOCAL_HARNESS_COMPLETE_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_all_domain_local_harness_complete(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<AllDomainLocalHarnessCompleteReport> {
    render_harness_with_dependencies(repo_root, output_path, None)
}

pub(crate) struct HarnessReportDependencies<'a> {
    pub(crate) stage_tools: Option<&'a AllDomainStageToolTableReport>,
    pub(crate) expected_results: Option<&'a AllDomainExpectedBenchmarkResultsReport>,
    pub(crate) commands: Option<&'a AllDomainRenderedCommandsReport>,
    pub(crate) outputs: Option<&'a AllDomainOutputDeclarationsReport>,
    pub(crate) parser: Option<&'a AllDomainParserCollectorReport>,
}

pub(crate) fn render_harness_with_dependencies(
    repo_root: &Path,
    output_path: PathBuf,
    dependencies: Option<HarnessReportDependencies<'_>>,
) -> Result<AllDomainLocalHarnessCompleteReport> {
    let _cwd_guard = CurrentDirGuard::change_to(repo_root);
    let absolute_output_path = repo_relative_path(repo_root, &output_path);
    if let Some(parent) = absolute_output_path.parent() {
        std::fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    let mut checks = Vec::new();

    let mut all_domain_stage_count = 0usize;
    let mut benchmark_ready_binding_count = 0usize;
    let mut expected_result_row_count = 0usize;
    let mut rendered_command_row_count = 0usize;
    let mut output_declaration_row_count = 0usize;
    let mut fake_run_result_count = 0usize;
    let mut fake_run_output_count = 0usize;
    let mut fake_failure_result_count = 0usize;
    let mut fake_failure_output_count = 0usize;
    let mut completion_row_count = 0usize;
    let mut parser_collector_row_count = 0usize;
    let mut missing_result_row_count = 0usize;
    let mut failure_class_count = 0usize;
    let mut real_smoke_execution_count = 0usize;

    let mut stage_tool_report = None;
    let mut expected_results_report = None;
    let mut rendered_commands_report = None;
    let mut output_declarations_report = None;
    let mut fake_runs_report = None;
    let mut fake_failures_report = None;
    let mut completion_check_report = None;
    let mut parser_collector_report = None;
    let mut missing_result_report = None;

    record_goal_check(
        &mut checks,
        278,
        "all-domain stage inventory",
        Some(DEFAULT_ALL_DOMAIN_STAGE_LIST_PATH.to_string()),
        || {
            let report = render_all_domain_stage_inventory(
                repo_root,
                &[BenchLocalDomain::Fastq, BenchLocalDomain::Bam, BenchLocalDomain::Vcf],
                PathBuf::from(DEFAULT_ALL_DOMAIN_STAGE_LIST_PATH),
            )?;
            if report.total_stage_count != 71
                || report.selected_domains != ["fastq", "bam", "vcf"]
                || report.domain_counts.get("fastq").copied() != Some(27)
                || report.domain_counts.get("bam").copied() != Some(24)
                || report.domain_counts.get("vcf").copied() != Some(20)
            {
                bail!("all-domain stage inventory drifted from the governed 71-stage slice");
            }
            all_domain_stage_count = report.total_stage_count;
            Ok("validated the governed 71-stage FASTQ/BAM/VCF local inventory".to_string())
        },
    );

    record_goal_check(
        &mut checks,
        279,
        "all-domain stage tool table",
        Some(DEFAULT_ALL_DOMAIN_STAGE_TOOL_TABLE_PATH.to_string()),
        || {
            let report = acquire_dependency(
                dependencies.as_ref().map_or(ReportDependency::Generate, |inputs| {
                    ReportDependency::current(inputs.stage_tools)
                }),
                "stage tool table",
                || {
                    render_all_domain_stage_tool_table(
                        repo_root,
                        PathBuf::from(DEFAULT_ALL_DOMAIN_STAGE_TOOL_TABLE_PATH),
                    )
                },
            )?;
            if report.row_count != report.domain_counts.values().sum::<usize>()
                || report.benchmark_ready_row_count != report.benchmark_ready_unique_binding_count
                || report.benchmark_ready_unique_binding_count
                    != report.benchmark_ready_domain_counts.values().sum::<usize>()
                || ["fastq", "bam", "vcf"].iter().any(|domain| {
                    report.domain_counts.get(*domain).copied().unwrap_or_default() == 0
                        || report
                            .benchmark_ready_domain_counts
                            .get(*domain)
                            .copied()
                            .unwrap_or_default()
                            == 0
                })
            {
                bail!("all-domain stage tool table drifted from the governed binding set");
            }
            benchmark_ready_binding_count = report.benchmark_ready_unique_binding_count;
            stage_tool_report = Some(report);
            Ok(format!(
                "validated {benchmark_ready_binding_count} benchmark-ready all-domain stage/tool bindings"
            ))
        },
    );

    record_goal_check(
        &mut checks,
        280,
        "all-domain expected benchmark results",
        Some(DEFAULT_ALL_DOMAIN_EXPECTED_BENCHMARK_RESULTS_PATH.to_string()),
        || {
            let report = acquire_dependency(
                dependencies.as_ref().map_or(ReportDependency::Generate, |inputs| {
                    ReportDependency::current(inputs.expected_results)
                }),
                "expected results",
                || {
                    render_all_domain_expected_benchmark_results(
                        repo_root,
                        PathBuf::from(DEFAULT_ALL_DOMAIN_EXPECTED_BENCHMARK_RESULTS_PATH),
                    )
                },
            )?;
            let stage_tool_report = stage_tool_report
                .as_ref()
                .context("goal 279 stage tool table report is required")?;
            let stage_count =
                report.rows.iter().map(|row| row.stage_id.as_str()).collect::<BTreeSet<_>>().len();
            let tool_count =
                report.rows.iter().map(|row| row.tool_id.as_str()).collect::<BTreeSet<_>>().len();
            let corpus_count =
                report.rows.iter().map(|row| row.corpus_id.as_str()).collect::<BTreeSet<_>>().len();
            let asset_profile_count = report
                .rows
                .iter()
                .map(|row| row.asset_profile_id.as_str())
                .collect::<BTreeSet<_>>()
                .len();
            if report.row_count != benchmark_ready_binding_count
                || report.result_id_count != benchmark_ready_binding_count
                || report.stage_count != stage_count
                || report.tool_count != tool_count
                || report.corpus_count != corpus_count
                || report.asset_profile_count != asset_profile_count
                || report.domain_counts != stage_tool_report.benchmark_ready_domain_counts
            {
                bail!("all-domain expected benchmark results drifted from the governed slice");
            }
            expected_result_row_count = report.row_count;
            expected_results_report = Some(report);
            Ok(format!(
                "validated {expected_result_row_count} canonical all-domain expected benchmark results"
            ))
        },
    );

    record_goal_check(
        &mut checks,
        281,
        "all-domain rendered commands",
        Some(DEFAULT_ALL_DOMAIN_RENDERED_COMMANDS_PATH.to_string()),
        || {
            let report = acquire_dependency(
                dependencies.as_ref().map_or(ReportDependency::Generate, |inputs| {
                    ReportDependency::current(inputs.commands)
                }),
                "rendered commands",
                || {
                    render_all_domain_commands(
                        repo_root,
                        PathBuf::from(DEFAULT_ALL_DOMAIN_RENDERED_COMMANDS_PATH),
                    )
                },
            )?;
            let expected_results_report = expected_results_report
                .as_ref()
                .context("goal 280 expected benchmark results report is required")?;
            let expected_fastq_bam_command_count = ["fastq", "bam"]
                .iter()
                .filter_map(|domain| expected_results_report.domain_counts.get(*domain))
                .sum::<usize>();
            if report.row_count != benchmark_ready_binding_count
                || report.result_id_count != benchmark_ready_binding_count
                || report.domain_counts != expected_results_report.domain_counts
                || report.benchmark_status_counts.get("benchmark_ready").copied()
                    != Some(benchmark_ready_binding_count)
                || report.command_source_counts.get("fastq_bam_command_adapter").copied()
                    != Some(expected_fastq_bam_command_count)
                || report.command_source_counts.get("vcf_bcftools_adapter").copied() != Some(11)
                || report.command_source_counts.get("vcf_angsd_adapter").copied() != Some(2)
                || report.command_source_counts.get("vcf_eigensoft_adapter").copied() != Some(2)
                || report.command_source_counts.get("vcf_imputation_family_adapter").copied()
                    != Some(8)
                || report.command_source_counts.get("vcf_phasing_family_adapter").copied()
                    != Some(4)
                || report.command_source_counts.get("vcf_plink_family_adapter").copied() != Some(7)
                || report.command_source_counts.get("vcf_descent_family_adapter").copied()
                    != Some(5)
            {
                bail!("all-domain rendered commands drifted from the governed binding slice");
            }
            rendered_command_row_count = report.row_count;
            rendered_commands_report = Some(report);
            Ok(format!(
                "validated executable commands for all {rendered_command_row_count} benchmark-ready all-domain results"
            ))
        },
    );

    record_goal_check(
        &mut checks,
        282,
        "all-domain output declarations",
        Some(DEFAULT_ALL_DOMAIN_OUTPUT_DECLARATIONS_PATH.to_string()),
        || {
            let report = acquire_dependency(
                dependencies.as_ref().map_or(ReportDependency::Generate, |inputs| {
                    ReportDependency::current(inputs.outputs)
                }),
                "output declarations",
                || {
                    render_all_domain_output_declarations(
                        repo_root,
                        PathBuf::from(DEFAULT_ALL_DOMAIN_OUTPUT_DECLARATIONS_PATH),
                    )
                },
            )?;
            let expected_results_report = expected_results_report
                .as_ref()
                .context("goal 280 expected benchmark results report is required")?;
            if report.row_count != benchmark_ready_binding_count
                || report.result_id_count != benchmark_ready_binding_count
                || report.complete_row_count != benchmark_ready_binding_count
                || report.incomplete_row_count != 0
                || report.domain_counts != expected_results_report.domain_counts
                || report.status_counts.get("complete").copied()
                    != Some(benchmark_ready_binding_count)
            {
                bail!("all-domain output declarations drifted from the governed complete slice");
            }
            output_declaration_row_count = report.row_count;
            output_declarations_report = Some(report);
            Ok(format!(
                "validated complete output declarations for all {output_declaration_row_count} benchmark-ready results"
            ))
        },
    );

    record_goal_check(
        &mut checks,
        283,
        "all-domain fake-runner",
        Some(DEFAULT_ALL_DOMAIN_FAKE_RUN_ROOT.to_string()),
        || {
            let expected_results_report = expected_results_report
                .as_ref()
                .context("goal 280 expected benchmark results report is required")?;
            let commands = rendered_commands_report
                .as_ref()
                .context("goal 281 rendered commands report is required")?;
            let outputs = output_declarations_report
                .as_ref()
                .context("goal 282 output declarations report is required")?;
            let report = bijux_dna_api::v1::api::bench::fake_run_all_domain_benchmark_results(
                repo_root,
                Path::new(DEFAULT_ALL_DOMAIN_FAKE_RUN_ROOT),
                &expected_results_report.rows,
                &commands.rows,
                &outputs.rows,
            )?;
            if report.result_count != benchmark_ready_binding_count
                || report.created_output_count == 0
                || report.domain_counts != expected_results_report.domain_counts
            {
                bail!("all-domain fake-runner drifted from the governed result slice");
            }
            fake_run_result_count = report.result_count;
            fake_run_output_count = report.created_output_count;
            fake_runs_report = Some(report);
            Ok(format!(
                "validated fake-run materialization for all {fake_run_result_count} benchmark-ready results"
            ))
        },
    );

    record_goal_check(
        &mut checks,
        284,
        "all-domain fake failures",
        Some(DEFAULT_ALL_DOMAIN_FAKE_FAILURE_ROOT.to_string()),
        || {
            let expected_results_report = expected_results_report
                .as_ref()
                .context("goal 280 expected benchmark results report is required")?;
            let commands = rendered_commands_report
                .as_ref()
                .context("goal 281 rendered commands report is required")?;
            let outputs = output_declarations_report
                .as_ref()
                .context("goal 282 output declarations report is required")?;
            let report = bijux_dna_api::v1::api::bench::fake_run_all_domain_failure_results(
                repo_root,
                Path::new(DEFAULT_ALL_DOMAIN_FAKE_FAILURE_ROOT),
                &expected_results_report.rows,
                &commands.rows,
                &outputs.rows,
                7,
            )?;
            if report.result_count != benchmark_ready_binding_count
                || report.failed_output_count == 0
                || report.exit_code != 7
                || report.domain_counts != expected_results_report.domain_counts
            {
                bail!("all-domain fake-failure runner drifted from the governed result slice");
            }
            fake_failure_result_count = report.result_count;
            fake_failure_output_count = report.failed_output_count;
            fake_failures_report = Some(report);
            Ok(format!(
                "validated structured failure records for all {fake_failure_result_count} benchmark-ready results"
            ))
        },
    );

    record_goal_check(
        &mut checks,
        285,
        "all-domain completion check",
        Some(DEFAULT_ALL_DOMAIN_COMPLETION_CHECK_PATH.to_string()),
        || {
            let expected_results_report = expected_results_report
                .as_ref()
                .context("goal 280 expected benchmark results report is required")?;
            let commands = rendered_commands_report
                .as_ref()
                .context("goal 281 rendered commands report is required")?;
            let outputs = output_declarations_report
                .as_ref()
                .context("goal 282 output declarations report is required")?;
            let report = bijux_dna_api::v1::api::bench::render_all_domain_completion_check(
                repo_root,
                Path::new(DEFAULT_ALL_DOMAIN_COMPLETION_CHECK_PATH),
                &expected_results_report.rows,
                &commands.rows,
                &outputs.rows,
            )?;
            if report.row_count != benchmark_ready_binding_count
                || report.complete_row_count + report.incomplete_row_count != report.row_count
                || report.incomplete_row_count != 5
                || !report.passes_behavior_test
                || report.domain_counts != expected_results_report.domain_counts
            {
                bail!("all-domain completion checker drifted from the governed seeded behavior");
            }
            completion_row_count = report.row_count;
            completion_check_report = Some(report);
            Ok(format!(
                "validated governed completion behavior across the {completion_row_count}-result harness slice"
            ))
        },
    );

    record_goal_check(
        &mut checks,
        286,
        "all-domain parser collector",
        Some(DEFAULT_ALL_DOMAIN_PARSER_COLLECTOR_PATH.to_string()),
        || {
            let report = acquire_dependency(
                dependencies.as_ref().map_or(ReportDependency::Generate, |inputs| {
                    ReportDependency::current(inputs.parser)
                }),
                "parser collector",
                || {
                    render_all_domain_parser_collector(
                        repo_root,
                        PathBuf::from(DEFAULT_ALL_DOMAIN_PARSER_COLLECTOR_PATH),
                    )
                },
            )?;
            let domain_counts_match_rows = ["fastq", "bam", "vcf"].iter().all(|domain| {
                let expected_count = expected_results_report
                    .as_ref()
                    .and_then(|expected| expected.domain_counts.get(*domain))
                    .copied()
                    .unwrap_or_default()
                    + report
                        .rows
                        .iter()
                        .filter(|row| {
                            row.domain == *domain
                                && row.source_kind == AllDomainParserCollectorSourceKind::RealSmoke
                        })
                        .count();
                report.domain_counts.get(*domain).copied() == Some(expected_count)
            });
            if report.row_count != report.fake_run_row_count + report.real_smoke_row_count
                || report.fake_run_row_count != benchmark_ready_binding_count
                || report.real_smoke_row_count != 4
                || !domain_counts_match_rows
            {
                bail!(
                    "all-domain parser collector drifted from the governed fake-run and smoke set"
                );
            }
            parser_collector_row_count = report.row_count;
            parser_collector_report = Some(report);
            Ok(format!(
                "validated parser collection for {benchmark_ready_binding_count} fake-run rows and 4 governed real-smoke rows"
            ))
        },
    );

    record_goal_check(
        &mut checks,
        287,
        "all-domain missing-result test",
        Some(DEFAULT_ALL_DOMAIN_MISSING_RESULT_TEST_PATH.to_string()),
        || {
            let expected_results_report = expected_results_report
                .as_ref()
                .context("goal 280 expected benchmark results report is required")?;
            let commands = rendered_commands_report
                .as_ref()
                .context("goal 281 rendered commands report is required")?;
            let outputs = output_declarations_report
                .as_ref()
                .context("goal 282 output declarations report is required")?;
            let report = bijux_dna_api::v1::api::bench::render_all_domain_missing_result_audit(
                repo_root,
                Path::new(DEFAULT_ALL_DOMAIN_MISSING_RESULT_TEST_PATH),
                &expected_results_report.rows,
                &commands.rows,
                &outputs.rows,
            )?;
            if report.expected_row_count != benchmark_ready_binding_count
                || report.missing_result_row_count != 3
                || report.present_result_row_count + report.missing_result_row_count
                    != report.expected_row_count
                || !report.passes_behavior_test
                || report.domain_counts != expected_results_report.domain_counts
            {
                bail!("all-domain missing-result behavior drifted from the governed 3-row probe");
            }
            missing_result_row_count = report.missing_result_row_count;
            missing_result_report = Some(report);
            Ok(format!(
                "validated one explicit missing result row per domain across the {benchmark_ready_binding_count}-result slice"
            ))
        },
    );

    record_goal_check(
        &mut checks,
        288,
        "all-domain failure classification",
        Some(DEFAULT_ALL_DOMAIN_FAILURE_CLASSIFICATION_PATH.to_string()),
        || {
            let report = render_all_domain_failure_classification(
                repo_root,
                PathBuf::from(DEFAULT_ALL_DOMAIN_FAILURE_CLASSIFICATION_PATH),
            )?;
            if report.row_count != 8
                || report.triggered_row_count != 8
                || report.required_class_count != 8
                || report.triggered_class_count != 8
                || report.missing_class_count != 0
                || !report.passes_behavior_test
            {
                bail!("all-domain failure classification drifted from the governed class set");
            }
            failure_class_count = report.triggered_class_count;
            Ok("validated all 8 required all-domain failure classes".to_string())
        },
    );

    record_goal_check(
        &mut checks,
        289,
        "all-domain real-smoke subset",
        Some(DEFAULT_REAL_SMOKE_CORE_SUBSET_PATH.to_string()),
        || {
            let report = render_real_smoke_core_subset(
                repo_root,
                PathBuf::from(DEFAULT_REAL_SMOKE_CORE_SUBSET_PATH),
            )?;
            if report.execution_count != 4
                || report.stage_execution_count != 3
                || report.pipeline_bridge_count != 1
                || !report.passes_behavior_test
                || report.domain_counts.get("fastq").copied() != Some(1)
                || report.domain_counts.get("bam").copied() != Some(1)
                || report.domain_counts.get("vcf").copied() != Some(2)
            {
                bail!("all-domain real-smoke subset drifted from the governed 4-execution slice");
            }
            real_smoke_execution_count = report.execution_count;
            Ok("validated real FASTQ, BAM, VCF, and bam-to-vcf bridge smoke execution".to_string())
        },
    );

    if let (
        Some(stage_tool_report),
        Some(expected_results_report),
        Some(rendered_commands_report),
        Some(output_declarations_report),
        Some(fake_runs_report),
        Some(fake_failures_report),
        Some(completion_check_report),
        Some(parser_collector_report),
        Some(missing_result_report),
    ) = (
        stage_tool_report.as_ref(),
        expected_results_report.as_ref(),
        rendered_commands_report.as_ref(),
        output_declarations_report.as_ref(),
        fake_runs_report.as_ref(),
        fake_failures_report.as_ref(),
        completion_check_report.as_ref(),
        parser_collector_report.as_ref(),
        missing_result_report.as_ref(),
    ) {
        if let Err(error) = validate_all_domain_harness_alignment(HarnessAlignmentEvidence {
            stage_tool_report,
            expected_results_report,
            rendered_commands_report,
            output_declarations_report,
            fake_runs_report,
            fake_failures_report,
            completion_check_report,
            parser_collector_report,
            missing_result_report,
        }) {
            checks.push(AllDomainLocalHarnessCompleteGoalCheck {
                goal_id: 398,
                surface: "all-domain local harness alignment".to_string(),
                output_path: None,
                ok: false,
                detail: error.to_string(),
            });
        }
    }

    let passed_goal_count = checks.iter().filter(|check| check.ok).count();
    let failing_goal_ids =
        checks.iter().filter(|check| !check.ok).map(|check| check.goal_id).collect::<Vec<_>>();
    let failed_goal_count = failing_goal_ids.len();
    let report = AllDomainLocalHarnessCompleteReport {
        schema_version: ALL_DOMAIN_LOCAL_HARNESS_COMPLETE_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &absolute_output_path),
        checked_goal_count: checks.len(),
        passed_goal_count,
        failed_goal_count,
        failing_goal_ids,
        all_domain_stage_count,
        benchmark_ready_binding_count,
        expected_result_row_count,
        rendered_command_row_count,
        output_declaration_row_count,
        fake_run_result_count,
        fake_run_output_count,
        fake_failure_result_count,
        fake_failure_output_count,
        completion_row_count,
        parser_collector_row_count,
        missing_result_row_count,
        failure_class_count,
        real_smoke_execution_count,
        ok: failed_goal_count == 0,
        checks,
    };
    bijux_dna_infra::atomic_write_json(&absolute_output_path, &report)?;
    Ok(report)
}

enum ReportDependency<'a, T> {
    Generate,
    Ready(&'a T),
    Failed,
}
impl<'a, T> ReportDependency<'a, T> {
    fn current(report: Option<&'a T>) -> Self {
        report.map_or(Self::Failed, Self::Ready)
    }
}

fn acquire_dependency<'a, T: Clone>(
    prior: ReportDependency<'a, T>,
    label: &str,
    generate: impl FnOnce() -> Result<T>,
) -> Result<std::borrow::Cow<'a, T>> {
    match prior {
        ReportDependency::Generate => generate().map(std::borrow::Cow::Owned),
        ReportDependency::Ready(report) => Ok(std::borrow::Cow::Borrowed(report)),
        ReportDependency::Failed => {
            bail!("current {label} dependency failed before harness collection")
        }
    }
}

fn record_goal_check<F>(
    checks: &mut Vec<AllDomainLocalHarnessCompleteGoalCheck>,
    goal_id: u32,
    surface: impl Into<String>,
    output_path: Option<String>,
    check: F,
) where
    F: FnOnce() -> Result<String>,
{
    let surface = surface.into();
    match check() {
        Ok(detail) => checks.push(AllDomainLocalHarnessCompleteGoalCheck {
            goal_id,
            surface,
            output_path,
            ok: true,
            detail,
        }),
        Err(error) => checks.push(AllDomainLocalHarnessCompleteGoalCheck {
            goal_id,
            surface,
            output_path,
            ok: false,
            detail: error.to_string(),
        }),
    }
}

fn repo_relative_path(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}

struct CurrentDirGuard {
    previous: PathBuf,
}

impl CurrentDirGuard {
    fn change_to(path: &Path) -> Self {
        let previous = env::current_dir().expect("current dir");
        env::set_current_dir(path).expect("set current dir");
        Self { previous }
    }
}

impl Drop for CurrentDirGuard {
    fn drop(&mut self) {
        env::set_current_dir(&self.previous).expect("restore current dir");
    }
}

#[cfg(test)]
mod dependency_tests {
    use super::{acquire_dependency, ReportDependency};
    #[test]
    fn current_reports_are_borrowed_and_failed_dependencies_never_regenerated() -> anyhow::Result<()>
    {
        let calls = std::cell::Cell::new(0);
        let generated = || {
            calls.set(calls.get() + 1);
            Ok::<_, anyhow::Error>("generated".to_string())
        };
        let prior = "observed".to_string();
        let borrowed = acquire_dependency(ReportDependency::Ready(&prior), "report", generated)?;
        assert!(std::ptr::eq(borrowed.as_ref(), &raw const prior));
        assert_eq!(calls.get(), 0);
        assert!(
            acquire_dependency::<String>(ReportDependency::Failed, "report", generated).is_err()
        );
        assert_eq!(calls.get(), 0);
        assert_eq!(
            acquire_dependency(ReportDependency::Generate, "report", generated)?.as_ref(),
            "generated"
        );
        assert_eq!(calls.get(), 1);
        Ok(())
    }
}
