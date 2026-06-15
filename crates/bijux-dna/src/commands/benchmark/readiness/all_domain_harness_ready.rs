use std::collections::BTreeSet;
use std::env;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use serde::Serialize;

use super::all_domain_completion_check::{
    render_all_domain_completion_check, DEFAULT_ALL_DOMAIN_COMPLETION_CHECK_PATH,
};
use super::all_domain_expected_benchmark_results::{
    render_all_domain_expected_benchmark_results, AllDomainExpectedBenchmarkResultsReport,
    DEFAULT_ALL_DOMAIN_EXPECTED_BENCHMARK_RESULTS_PATH,
};
use super::all_domain_failure_classification::{
    render_all_domain_failure_classification, DEFAULT_ALL_DOMAIN_FAILURE_CLASSIFICATION_PATH,
};
use super::all_domain_missing_result_test::{
    render_all_domain_missing_result_test, DEFAULT_ALL_DOMAIN_MISSING_RESULT_TEST_PATH,
};
use super::all_domain_output_declarations::{
    render_all_domain_output_declarations, AllDomainOutputDeclarationsReport,
    DEFAULT_ALL_DOMAIN_OUTPUT_DECLARATIONS_PATH,
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
use crate::commands::benchmark::local_all_domain_fake_failures::{
    fake_run_all_domain_failures, DEFAULT_ALL_DOMAIN_FAKE_FAILURE_ROOT,
};
use crate::commands::benchmark::local_all_domain_fake_runs::{
    fake_run_all_domain_benchmark_results, DEFAULT_ALL_DOMAIN_FAKE_RUN_ROOT,
};
use crate::commands::benchmark::local_real_smoke_core_subset::{
    render_real_smoke_core_subset, DEFAULT_REAL_SMOKE_CORE_SUBSET_PATH,
};
use crate::commands::benchmark::local_stage_inventory::{
    render_all_domain_stage_inventory, BenchLocalDomain, DEFAULT_ALL_DOMAIN_STAGE_LIST_PATH,
};
use crate::commands::benchmark::local_stage_result_manifest::path_relative_to_repo;
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_ALL_DOMAIN_HARNESS_READY_PATH: &str =
    "benchmarks/readiness/ALL_DOMAIN_HARNESS_READY.json";
const ALL_DOMAIN_HARNESS_READY_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.all_domain_harness_ready.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct AllDomainHarnessReadyGoalCheck {
    pub(crate) goal_id: u32,
    pub(crate) surface: String,
    pub(crate) output_path: Option<String>,
    pub(crate) ok: bool,
    pub(crate) detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct HarnessBindingKey {
    domain: String,
    stage_id: String,
    tool_id: String,
    corpus_id: String,
    asset_profile_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AllDomainHarnessReadyReport {
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
    pub(crate) checks: Vec<AllDomainHarnessReadyGoalCheck>,
}

pub(crate) fn run_render_all_domain_harness_ready(
    args: &parse::BenchReadinessRenderAllDomainHarnessReadyArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_all_domain_harness_ready(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_ALL_DOMAIN_HARNESS_READY_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_all_domain_harness_ready(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<AllDomainHarnessReadyReport> {
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
            let report = render_all_domain_stage_tool_table(
                repo_root,
                PathBuf::from(DEFAULT_ALL_DOMAIN_STAGE_TOOL_TABLE_PATH),
            )?;
            if report.row_count != 146
                || report.benchmark_ready_row_count != report.benchmark_ready_unique_binding_count
                || report.domain_counts.get("fastq").copied() != Some(74)
                || report.domain_counts.get("bam").copied() != Some(49)
                || report.domain_counts.get("vcf").copied() != Some(23)
                || report.benchmark_ready_domain_counts.get("fastq").copied().unwrap_or_default()
                    == 0
                || report.benchmark_ready_domain_counts.get("bam").copied().unwrap_or_default() == 0
                || report.benchmark_ready_domain_counts.get("vcf").copied().unwrap_or_default() == 0
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
            let report = render_all_domain_expected_benchmark_results(
                repo_root,
                PathBuf::from(DEFAULT_ALL_DOMAIN_EXPECTED_BENCHMARK_RESULTS_PATH),
            )?;
            let stage_tool_report = stage_tool_report
                .as_ref()
                .context("goal 279 stage tool table report is required")?;
            if report.row_count != benchmark_ready_binding_count
                || report.result_id_count != benchmark_ready_binding_count
                || report.stage_count != 66
                || report.tool_count != 69
                || report.corpus_count != 9
                || report.asset_profile_count != 13
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
            let report = render_all_domain_commands(
                repo_root,
                PathBuf::from(DEFAULT_ALL_DOMAIN_RENDERED_COMMANDS_PATH),
            )?;
            let expected_results_report = expected_results_report
                .as_ref()
                .context("goal 280 expected benchmark results report is required")?;
            if report.row_count != benchmark_ready_binding_count
                || report.result_id_count != benchmark_ready_binding_count
                || report.domain_counts != expected_results_report.domain_counts
                || report.benchmark_status_counts.get("benchmark_ready").copied()
                    != Some(benchmark_ready_binding_count)
                || report.command_source_counts.get("fastq_bam_command_adapter").copied()
                    != Some(116)
                || report.command_source_counts.get("vcf_bcftools_adapter").copied() != Some(11)
                || report.command_source_counts.get("vcf_eigensoft_adapter").copied() != Some(1)
                || report.command_source_counts.get("vcf_imputation_family_adapter").copied()
                    != Some(2)
                || report.command_source_counts.get("vcf_phasing_family_adapter").copied()
                    != Some(1)
                || report.command_source_counts.get("vcf_plink_family_adapter").copied() != Some(5)
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
            let report = render_all_domain_output_declarations(
                repo_root,
                PathBuf::from(DEFAULT_ALL_DOMAIN_OUTPUT_DECLARATIONS_PATH),
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
            let report = fake_run_all_domain_benchmark_results(
                repo_root,
                PathBuf::from(DEFAULT_ALL_DOMAIN_FAKE_RUN_ROOT),
            )?;
            let expected_results_report = expected_results_report
                .as_ref()
                .context("goal 280 expected benchmark results report is required")?;
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
            let report = fake_run_all_domain_failures(
                repo_root,
                PathBuf::from(DEFAULT_ALL_DOMAIN_FAKE_FAILURE_ROOT),
                7,
            )?;
            let expected_results_report = expected_results_report
                .as_ref()
                .context("goal 280 expected benchmark results report is required")?;
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
            let report = render_all_domain_completion_check(
                repo_root,
                PathBuf::from(DEFAULT_ALL_DOMAIN_COMPLETION_CHECK_PATH),
            )?;
            let expected_results_report = expected_results_report
                .as_ref()
                .context("goal 280 expected benchmark results report is required")?;
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
            let report = render_all_domain_parser_collector(
                repo_root,
                PathBuf::from(DEFAULT_ALL_DOMAIN_PARSER_COLLECTOR_PATH),
            )?;
            if report.row_count != report.fake_run_row_count + report.real_smoke_row_count
                || report.fake_run_row_count != benchmark_ready_binding_count
                || report.real_smoke_row_count != 4
                || report.domain_counts.get("fastq").copied() != Some(68)
                || report.domain_counts.get("bam").copied() != Some(50)
                || report.domain_counts.get("vcf").copied() != Some(22)
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
            let report = render_all_domain_missing_result_test(
                repo_root,
                PathBuf::from(DEFAULT_ALL_DOMAIN_MISSING_RESULT_TEST_PATH),
            )?;
            let expected_results_report = expected_results_report
                .as_ref()
                .context("goal 280 expected benchmark results report is required")?;
            if report.expected_row_count != benchmark_ready_binding_count
                || report.missing_result_row_count != 3
                || report.present_result_row_count + report.missing_result_row_count
                    != report.expected_row_count
                || !report.passes_behavior_test
                || report.domain_counts != expected_results_report.domain_counts
            {
                bail!("all-domain missing-result behavior drifted from the governed 3-row probe");
            }
            missing_result_row_count = report.expected_row_count;
            missing_result_report = Some(report);
            Ok(format!(
                "validated one explicit missing result row per domain across the {missing_result_row_count}-result slice"
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
            if report.row_count != 7
                || report.triggered_row_count != 7
                || report.required_class_count != 7
                || report.triggered_class_count != 7
                || report.missing_class_count != 0
                || !report.passes_behavior_test
            {
                bail!("all-domain failure classification drifted from the governed class set");
            }
            failure_class_count = report.triggered_class_count;
            Ok("validated all 7 required all-domain failure classes".to_string())
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

    let stage_tool_report = stage_tool_report
        .ok_or_else(|| anyhow!("goal 279 stage tool table report was not captured"))?;
    let expected_results_report = expected_results_report
        .ok_or_else(|| anyhow!("goal 280 expected benchmark results report was not captured"))?;
    let rendered_commands_report = rendered_commands_report
        .ok_or_else(|| anyhow!("goal 281 rendered commands report was not captured"))?;
    let output_declarations_report = output_declarations_report
        .ok_or_else(|| anyhow!("goal 282 output declarations report was not captured"))?;
    let fake_runs_report =
        fake_runs_report.ok_or_else(|| anyhow!("goal 283 fake-run report was not captured"))?;
    let fake_failures_report = fake_failures_report
        .ok_or_else(|| anyhow!("goal 284 fake-failure report was not captured"))?;
    let completion_check_report = completion_check_report
        .ok_or_else(|| anyhow!("goal 285 completion report was not captured"))?;
    let parser_collector_report = parser_collector_report
        .ok_or_else(|| anyhow!("goal 286 parser collector report was not captured"))?;
    let missing_result_report = missing_result_report
        .ok_or_else(|| anyhow!("goal 287 missing-result report was not captured"))?;

    validate_all_domain_harness_alignment(
        &stage_tool_report,
        &expected_results_report,
        &rendered_commands_report,
        &output_declarations_report,
        &fake_runs_report,
        &fake_failures_report,
        &completion_check_report,
        &parser_collector_report,
        &missing_result_report,
    )?;

    let passed_goal_count = checks.iter().filter(|check| check.ok).count();
    let failing_goal_ids =
        checks.iter().filter(|check| !check.ok).map(|check| check.goal_id).collect::<Vec<_>>();
    let failed_goal_count = failing_goal_ids.len();
    let report = AllDomainHarnessReadyReport {
        schema_version: ALL_DOMAIN_HARNESS_READY_SCHEMA_VERSION,
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

fn validate_all_domain_harness_alignment(
    stage_tool_report: &AllDomainStageToolTableReport,
    expected_results_report: &AllDomainExpectedBenchmarkResultsReport,
    rendered_commands_report: &AllDomainRenderedCommandsReport,
    output_declarations_report: &AllDomainOutputDeclarationsReport,
    fake_runs_report: &crate::commands::benchmark::local_all_domain_fake_runs::AllDomainFakeRunsReport,
    fake_failures_report: &crate::commands::benchmark::local_all_domain_fake_failures::AllDomainFakeFailuresManifest,
    completion_check_report: &crate::commands::benchmark::readiness::all_domain_completion_check::AllDomainCompletionCheckReport,
    parser_collector_report: &AllDomainParserCollectorReport,
    missing_result_report: &crate::commands::benchmark::readiness::all_domain_missing_result_test::AllDomainMissingResultTestReport,
) -> Result<()> {
    let expected_result_ids = expected_results_report
        .rows
        .iter()
        .map(|row| row.result_id.clone())
        .collect::<BTreeSet<_>>();
    let rendered_result_ids = rendered_commands_report
        .rows
        .iter()
        .map(|row| row.result_id.clone())
        .collect::<BTreeSet<_>>();
    let output_result_ids = output_declarations_report
        .rows
        .iter()
        .map(|row| row.result_id.clone())
        .collect::<BTreeSet<_>>();
    let fake_run_result_ids =
        fake_runs_report.results.iter().map(|row| row.result_id.clone()).collect::<BTreeSet<_>>();
    let fake_failure_result_ids = fake_failures_report
        .failures
        .iter()
        .map(|row| row.result_id.clone())
        .collect::<BTreeSet<_>>();
    let completion_result_ids = completion_check_report
        .rows
        .iter()
        .map(|row| row.result_id.clone())
        .collect::<BTreeSet<_>>();
    let missing_result_ids =
        missing_result_report.rows.iter().map(|row| row.result_id.clone()).collect::<BTreeSet<_>>();

    ensure_matching_set("rendered commands", &expected_result_ids, &rendered_result_ids)?;
    ensure_matching_set("output declarations", &expected_result_ids, &output_result_ids)?;
    ensure_matching_set("fake runs", &expected_result_ids, &fake_run_result_ids)?;
    ensure_matching_set("fake failures", &expected_result_ids, &fake_failure_result_ids)?;
    ensure_matching_set("completion rows", &expected_result_ids, &completion_result_ids)?;
    ensure_matching_set("missing-result rows", &expected_result_ids, &missing_result_ids)?;

    let parser_fake_run_result_ids = parser_collector_report
        .rows
        .iter()
        .filter_map(|row| {
            (row.source_kind
                == crate::commands::benchmark::readiness::all_domain_parser_collector::AllDomainParserCollectorSourceKind::FakeRun)
                .then(|| row.result_id.clone())
                .flatten()
        })
        .collect::<BTreeSet<_>>();
    ensure_matching_set(
        "parser collector fake-run rows",
        &expected_result_ids,
        &parser_fake_run_result_ids,
    )?;

    let expected_bindings =
        expected_results_report.rows.iter().map(expected_binding_key).collect::<BTreeSet<_>>();
    let stage_tool_bindings = stage_tool_report
        .rows
        .iter()
        .filter(|row| row.benchmark_status == "benchmark_ready")
        .map(stage_tool_binding_key)
        .collect::<BTreeSet<_>>();
    ensure_matching_binding_set("stage tool table", &expected_bindings, &stage_tool_bindings)?;

    Ok(())
}

fn ensure_matching_set(
    label: &str,
    expected: &BTreeSet<String>,
    observed: &BTreeSet<String>,
) -> Result<()> {
    if expected != observed {
        let missing = expected.difference(observed).cloned().collect::<Vec<_>>();
        let extra = observed.difference(expected).cloned().collect::<Vec<_>>();
        bail!("{label} drifted from the canonical all-domain result slice; missing={missing:?} extra={extra:?}");
    }
    Ok(())
}

fn ensure_matching_binding_set(
    label: &str,
    expected: &BTreeSet<HarnessBindingKey>,
    observed: &BTreeSet<HarnessBindingKey>,
) -> Result<()> {
    if expected != observed {
        let missing = expected.difference(observed).cloned().collect::<Vec<_>>();
        let extra = observed.difference(expected).cloned().collect::<Vec<_>>();
        bail!("{label} drifted from the canonical all-domain binding slice; missing={missing:?} extra={extra:?}");
    }
    Ok(())
}

fn expected_binding_key(
    row: &crate::commands::benchmark::readiness::all_domain_expected_benchmark_results::AllDomainExpectedBenchmarkResultRow,
) -> HarnessBindingKey {
    HarnessBindingKey {
        domain: row.domain.clone(),
        stage_id: row.stage_id.clone(),
        tool_id: row.tool_id.clone(),
        corpus_id: row.corpus_id.clone(),
        asset_profile_id: row.asset_profile_id.clone(),
    }
}

fn stage_tool_binding_key(
    row: &crate::commands::benchmark::readiness::all_domain_stage_tool_table::AllDomainStageToolTableRow,
) -> HarnessBindingKey {
    HarnessBindingKey {
        domain: row.domain.clone(),
        stage_id: row.stage_id.clone(),
        tool_id: row.tool_id.clone(),
        corpus_id: row.corpus_id.clone(),
        asset_profile_id: row.asset_profile_id.clone(),
    }
}

fn record_goal_check<F>(
    checks: &mut Vec<AllDomainHarnessReadyGoalCheck>,
    goal_id: u32,
    surface: impl Into<String>,
    output_path: Option<String>,
    check: F,
) where
    F: FnOnce() -> Result<String>,
{
    let surface = surface.into();
    match check() {
        Ok(detail) => checks.push(AllDomainHarnessReadyGoalCheck {
            goal_id,
            surface,
            output_path,
            ok: true,
            detail,
        }),
        Err(error) => checks.push(AllDomainHarnessReadyGoalCheck {
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
