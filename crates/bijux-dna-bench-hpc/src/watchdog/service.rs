use super::completion_rules::build_completion_rules_report;
use super::failure_isolation::build_failure_isolation_report;
use super::no_global_wait::build_no_global_wait_report;
use super::partial_resume::build_partial_resume_report;
use super::{LocalDagWatchdogScenario, LocalDagWatchdogSimulationReport};
use anyhow::{Context, Result};
use std::path::Path;
/// Simulate one watchdog policy over admitted DAG evidence and atomically publish its report.
///
/// # Errors
/// Rejects invalid topology, unmet scenario conditions or failed report publication.
pub fn simulate_dag_watchdog_path(
    repo_root: &Path,
    config_path: &Path,
    dag_report_path: &Path,
    dag_report: &bijux_dna_bench_model::BenchmarkLocalPipelineDagValidationReport,
    scenario: LocalDagWatchdogScenario,
    output_path: &Path,
) -> Result<LocalDagWatchdogSimulationReport> {
    super::admission::admit(dag_report)?;
    let report = match scenario {
        LocalDagWatchdogScenario::NoGlobalWait => build_no_global_wait_report(
            repo_root,
            config_path,
            dag_report_path,
            output_path,
            dag_report,
        )?,
        LocalDagWatchdogScenario::FailureIsolation => build_failure_isolation_report(
            repo_root,
            config_path,
            dag_report_path,
            output_path,
            dag_report,
        )?,
        LocalDagWatchdogScenario::PartialResume => build_partial_resume_report(
            repo_root,
            config_path,
            dag_report_path,
            output_path,
            dag_report,
        )?,
        LocalDagWatchdogScenario::CompletionRules => build_completion_rules_report(
            repo_root,
            config_path,
            dag_report_path,
            output_path,
            dag_report,
        )?,
    };

    bijux_dna_infra::atomic_write_json(output_path, &report)
        .with_context(|| format!("write {}", output_path.display()))?;
    Ok(report)
}
