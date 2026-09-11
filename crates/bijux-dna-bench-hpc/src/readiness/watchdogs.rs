use crate::readiness::assessment::fail_check;
use crate::readiness::assessment::ok_check;
use crate::readiness::context::ReadinessContext;
use crate::watchdog::LocalDagWatchdogScenario;
use crate::watchdog::LocalDagWatchdogSimulationReport;
use crate::watchdog::DEFAULT_COMPLETION_RULES_REPORT_PATH;
use crate::watchdog::DEFAULT_FAILURE_ISOLATION_REPORT_PATH;
use crate::watchdog::DEFAULT_NO_GLOBAL_WAIT_REPORT_PATH;
use crate::watchdog::DEFAULT_PARTIAL_RESUME_REPORT_PATH;
use bijux_dna_bench_model::BenchmarkLocalHpcSubmissionReadinessGoalCheck as BenchLocalHpcSubmissionReadyGoalCheck;

pub(in crate::readiness) fn evaluate_watchdog_goals(
    context: &ReadinessContext<'_>,
    checks: &mut Vec<BenchLocalHpcSubmissionReadyGoalCheck>,
) {
    evaluate_watchdog_goal(
        context,
        checks,
        87,
        LocalDagWatchdogScenario::NoGlobalWait,
        DEFAULT_NO_GLOBAL_WAIT_REPORT_PATH,
        |report| report.no_global_wait_proven,
        "no-global-wait watchdog simulation keeps ready work moving while an unrelated branch is slow",
    );
    evaluate_watchdog_goal(
        context,
        checks,
        88,
        LocalDagWatchdogScenario::FailureIsolation,
        DEFAULT_FAILURE_ISOLATION_REPORT_PATH,
        |report| report.failure_isolation_proven,
        "failure-isolation watchdog simulation blocks only dependent work for the failed sample",
    );
    evaluate_watchdog_goal(
        context,
        checks,
        89,
        LocalDagWatchdogScenario::PartialResume,
        DEFAULT_PARTIAL_RESUME_REPORT_PATH,
        |report| report.partial_resume_proven,
        "partial-resume watchdog simulation reuses valid nodes and replans only invalid or missing work",
    );
    evaluate_watchdog_goal(
        context,
        checks,
        90,
        LocalDagWatchdogScenario::CompletionRules,
        DEFAULT_COMPLETION_RULES_REPORT_PATH,
        |report| report.completion_rules_proven,
        "completion-rules watchdog simulation requires zero exit, outputs, and result manifest together",
    );
}

pub(in crate::readiness) fn evaluate_watchdog_goal<F>(
    context: &ReadinessContext<'_>,
    checks: &mut Vec<BenchLocalHpcSubmissionReadyGoalCheck>,
    goal_id: u32,
    scenario: LocalDagWatchdogScenario,
    output_path: &str,
    predicate: F,
    surface: &str,
) where
    F: Fn(&LocalDagWatchdogSimulationReport) -> bool,
{
    let repo_root = context.repo_root;
    match context.backend.simulate_dag_watchdog_path(
        repo_root,
        scenario,
        &repo_root.join(output_path),
    ) {
        Ok(report) if predicate(&report) => checks.push(ok_check(
            goal_id,
            "pipeline_dags",
            surface,
            Some(report.output_path.clone()),
            format!("watchdog scenario `{}` produced a governed proof report", report.scenario),
        )),
        Ok(report) => checks.push(fail_check(
            goal_id,
            "pipeline_dags",
            surface,
            Some(report.output_path.clone()),
            format!("watchdog scenario `{}` did not prove the required invariant", report.scenario),
        )),
        Err(err) => checks.push(fail_check(
            goal_id,
            "pipeline_dags",
            surface,
            Some(output_path.to_string()),
            format!("{err:#}"),
        )),
    }
}
