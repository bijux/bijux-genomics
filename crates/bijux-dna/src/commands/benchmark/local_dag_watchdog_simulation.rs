use crate::commands::benchmark::local_pipeline_dag::{
    validate_pipeline_dag_path, DEFAULT_FASTQ_CORE_PREPROCESS_PIPELINE_CONFIG_PATH,
};
use crate::commands::cli::{parse, render};
use anyhow::{Context, Result};
pub(crate) use bijux_dna_bench_hpc::watchdog::{
    LocalDagWatchdogScenario, LocalDagWatchdogSimulationReport,
};
use std::path::Path;
const FASTQ_CORE_PREPROCESS_PIPELINE_REPORT_PATH: &str =
    "artifacts/benchmark-smoke/readiness/local-ready/pipeline-dag/fastq-core-preprocess.json";
pub(crate) fn simulate_dag_watchdog_path(
    repo_root: &Path,
    scenario: LocalDagWatchdogScenario,
    output_path: &Path,
) -> Result<LocalDagWatchdogSimulationReport> {
    let config_path = repo_root.join(DEFAULT_FASTQ_CORE_PREPROCESS_PIPELINE_CONFIG_PATH);
    let dag_report_path = repo_root.join(FASTQ_CORE_PREPROCESS_PIPELINE_REPORT_PATH);
    let dag_report = validate_pipeline_dag_path(repo_root, &config_path, &dag_report_path)?;

    bijux_dna_bench_hpc::watchdog::simulate_dag_watchdog_path(
        repo_root,
        &config_path,
        &dag_report_path,
        &dag_report,
        scenario,
        output_path,
    )
}

pub(crate) fn run_simulate_dag_watchdog(
    args: &parse::BenchLocalSimulateDagWatchdogArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let scenario = match args.scenario {
        parse::BenchLocalDagWatchdogScenarioArg::NoGlobalWait => {
            LocalDagWatchdogScenario::NoGlobalWait
        }
        parse::BenchLocalDagWatchdogScenarioArg::FailureIsolation => {
            LocalDagWatchdogScenario::FailureIsolation
        }
        parse::BenchLocalDagWatchdogScenarioArg::PartialResume => {
            LocalDagWatchdogScenario::PartialResume
        }
        parse::BenchLocalDagWatchdogScenarioArg::CompletionRules => {
            LocalDagWatchdogScenario::CompletionRules
        }
    };
    let output_path = match &args.output {
        Some(path) if path.is_absolute() => path.clone(),
        Some(path) => repo_root.join(path),
        None => repo_root.join(scenario.default_output_relative_path()),
    };
    let report = simulate_dag_watchdog_path(&repo_root, scenario, &output_path)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}
#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::{simulate_dag_watchdog_path, LocalDagWatchdogScenario};
    use bijux_dna_bench_hpc::watchdog::{
        DEFAULT_COMPLETION_RULES_REPORT_PATH, DEFAULT_FAILURE_ISOLATION_REPORT_PATH,
        DEFAULT_NO_GLOBAL_WAIT_REPORT_PATH, DEFAULT_PARTIAL_RESUME_REPORT_PATH,
    };

    fn repo_root() -> std::path::PathBuf {
        crate::commands::support::workspace_root::resolve_repo_root().expect("repo root")
    }

    #[test]
    fn no_global_wait_simulation_proves_ready_nodes_do_not_wait_for_a_slow_branch() {
        let repo_root = repo_root();
        let output_path = repo_root.join(DEFAULT_NO_GLOBAL_WAIT_REPORT_PATH);
        let report = simulate_dag_watchdog_path(
            &repo_root,
            LocalDagWatchdogScenario::NoGlobalWait,
            &output_path,
        )
        .expect("simulate no-global-wait watchdog report");

        assert_eq!(report.scenario, "no_global_wait");
        assert_eq!(report.pipeline_id, "fastq-core-preprocess");
        assert_eq!(report.node_count, 7);
        assert_eq!(report.slow_branch_stage_id, "fastq.profile_read_lengths");
        assert_eq!(report.slow_branch_finish_second, 13);
        assert!(report.no_global_wait_proven);
        assert!(
            report
                .ready_while_slow_branch_running_stage_ids
                .iter()
                .any(|stage_id| stage_id == "fastq.trim_reads"),
            "trim_reads must be allowed to start while the slow profiling branch is still running"
        );
        assert!(
            report
                .ready_while_slow_branch_running_stage_ids
                .iter()
                .any(|stage_id| stage_id == "fastq.filter_reads"),
            "filter_reads must stay unblocked by the unrelated slow profiling branch"
        );
        assert!(
            report.nodes.iter().any(|node| {
                node.stage_id == "fastq.trim_reads"
                    && node.start_second == 3
                    && node.finish_second == 4
            }),
            "trim_reads should start as soon as validate_reads and detect_adapters are done"
        );
        assert!(
            report.nodes.iter().any(|node| {
                node.stage_id == "fastq.profile_read_lengths"
                    && node.start_second == 1
                    && node.finish_second == 13
            }),
            "profile_read_lengths must remain the intentionally slow branch in the simulation"
        );
    }

    #[test]
    fn failure_isolation_simulation_keeps_unrelated_sample_work_running() {
        let repo_root = repo_root();
        let output_path = repo_root.join(DEFAULT_FAILURE_ISOLATION_REPORT_PATH);
        let report = simulate_dag_watchdog_path(
            &repo_root,
            LocalDagWatchdogScenario::FailureIsolation,
            &output_path,
        )
        .expect("simulate failure-isolation watchdog report");

        assert_eq!(report.scenario, "failure_isolation");
        assert_eq!(report.pipeline_id, "fastq-core-preprocess");
        assert_eq!(report.sample_count, 2);
        assert_eq!(report.node_count, 14);
        assert_eq!(report.failed_sample_id.as_deref(), Some("sample_alpha"));
        assert_eq!(report.failed_stage_id.as_deref(), Some("fastq.detect_adapters"));
        assert_eq!(report.failure_second, Some(3));
        assert!(report.failure_isolation_proven);
        assert!(
            report
                .continued_unrelated_node_ids
                .iter()
                .any(|node_id| node_id == "sample_beta::fastq.trim_reads"),
            "the unaffected sample must continue into trimming after the injected failure"
        );
        assert!(
            report
                .continued_unrelated_node_ids
                .iter()
                .any(|node_id| node_id == "sample_beta::fastq.report_qc"),
            "the unaffected sample must continue all the way to downstream QC collation"
        );
        assert!(
            report
                .blocked_node_ids
                .iter()
                .any(|node_id| node_id == "sample_alpha::fastq.trim_reads"),
            "the failed sample must block only its own dependent downstream work"
        );
        assert!(
            report.nodes.iter().any(|node| {
                node.node_id == "sample_alpha::fastq.detect_adapters"
                    && node.status == "failed"
                    && node.finish_second == 3
            }),
            "the injected failure must be recorded on the failed sample stage"
        );
        assert!(
            report.nodes.iter().any(|node| {
                node.node_id == "sample_beta::fastq.profile_reads"
                    && node.status == "completed"
                    && node.finish_second == 6
            }),
            "the unaffected sample must continue completing downstream work after the failure"
        );
    }

    #[test]
    fn partial_resume_simulation_reuses_valid_nodes_and_replans_only_missing_or_invalid_work() {
        let repo_root = repo_root();
        let output_path = repo_root.join(DEFAULT_PARTIAL_RESUME_REPORT_PATH);
        let report = simulate_dag_watchdog_path(
            &repo_root,
            LocalDagWatchdogScenario::PartialResume,
            &output_path,
        )
        .expect("simulate partial-resume watchdog report");

        assert_eq!(report.scenario, "partial_resume");
        assert_eq!(report.pipeline_id, "fastq-core-preprocess");
        assert_eq!(report.sample_count, 1);
        assert_eq!(report.node_count, 7);
        assert!(report.partial_resume_proven);
        assert_eq!(
            report.reused_valid_node_ids,
            vec![
                "fastq.validate_reads".to_string(),
                "fastq.profile_read_lengths".to_string(),
                "fastq.detect_adapters".to_string(),
            ]
        );
        assert_eq!(report.invalid_node_ids, vec!["fastq.trim_reads".to_string()]);
        assert_eq!(
            report.missing_node_ids,
            vec![
                "fastq.filter_reads".to_string(),
                "fastq.profile_reads".to_string(),
                "fastq.report_qc".to_string(),
            ]
        );
        assert_eq!(
            report.planned_node_ids,
            vec![
                "fastq.trim_reads".to_string(),
                "fastq.filter_reads".to_string(),
                "fastq.profile_reads".to_string(),
                "fastq.report_qc".to_string(),
            ]
        );
        assert!(
            report.nodes.iter().any(|node| {
                node.node_id == "fastq.detect_adapters"
                    && node.status == "reused"
                    && node.duration_seconds == 0
            }),
            "a valid completed upstream node must be reused without replanning"
        );
        assert!(
            report.nodes.iter().any(|node| {
                node.node_id == "fastq.trim_reads"
                    && node.status == "planned"
                    && node.start_second == 0
                    && node.finish_second == 1
            }),
            "the invalid node must be replanned"
        );
        assert!(
            report.nodes.iter().any(|node| {
                node.node_id == "fastq.report_qc"
                    && node.status == "planned"
                    && node.finish_second == 4
            }),
            "missing downstream work must be planned after the invalidated node path"
        );
    }

    #[test]
    fn completion_rules_simulation_requires_zero_exit_outputs_and_manifest() {
        let repo_root = repo_root();
        let output_path = repo_root.join(DEFAULT_COMPLETION_RULES_REPORT_PATH);
        let report = simulate_dag_watchdog_path(
            &repo_root,
            LocalDagWatchdogScenario::CompletionRules,
            &output_path,
        )
        .expect("simulate completion-rules watchdog report");

        assert_eq!(report.scenario, "completion_rules");
        assert_eq!(report.pipeline_id, "fastq-core-preprocess");
        assert_eq!(report.sample_count, 1);
        assert_eq!(report.node_count, 5);
        assert_eq!(report.completion_check_stage_id.as_deref(), Some("fastq.filter_reads"));
        assert!(report.completion_rules_proven);
        assert!(
            report.completion_checks.iter().any(|check| {
                check.case_id == "zero_exit_outputs_only"
                    && check.exit_code == 0
                    && check.declared_outputs_exist
                    && !check.result_manifest_exists
                    && !check.complete
            }),
            "zero exit with outputs alone must remain incomplete"
        );
        assert!(
            report.completion_checks.iter().any(|check| {
                check.case_id == "nonzero_exit_with_outputs_and_manifest"
                    && check.exit_code != 0
                    && check.declared_outputs_exist
                    && check.result_manifest_exists
                    && !check.complete
            }),
            "nonzero exit must refuse completion even if artifacts exist"
        );
        assert!(
            report.completion_checks.iter().any(|check| {
                check.case_id == "zero_exit_outputs_and_manifest"
                    && check.exit_code == 0
                    && check.declared_outputs_exist
                    && check.result_manifest_exists
                    && check.complete
            }),
            "completion should only be recorded when all governed requirements are satisfied"
        );
        assert!(
            report.nodes.iter().any(|node| {
                node.node_id == "sample_primary::fastq.filter_reads::zero_exit_outputs_only"
                    && node.status == "incomplete"
            }),
            "the outputs-only case must remain explicitly incomplete in the node report"
        );
    }
}
