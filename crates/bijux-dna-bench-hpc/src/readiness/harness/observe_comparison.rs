use crate::readiness::assessment::fail_check;
use crate::readiness::context::ReadinessContext;
use crate::readiness::declarations::DEFAULT_TOOL_COMPARISON_TEMPLATE_PATH;
use crate::readiness::harness::summary::evaluate_tool_comparison_goal;
use crate::readiness::ports::ToolComparisonObservation;
use bijux_dna_bench_model::BenchmarkLocalHpcSubmissionReadinessGoalCheck as BenchLocalHpcSubmissionReadyGoalCheck;
use std::path::PathBuf;

pub(super) fn observe_comparison(
    context: &ReadinessContext<'_>,
    checks: &mut Vec<BenchLocalHpcSubmissionReadyGoalCheck>,
) -> Option<ToolComparisonObservation> {
    let repo_root = context.repo_root;
    match context.backend.render_local_tool_comparison_template(
        repo_root,
        PathBuf::from(context.paths.fake_run_root),
        PathBuf::from(DEFAULT_TOOL_COMPARISON_TEMPLATE_PATH),
    ) {
        Ok(report) => {
            evaluate_tool_comparison_goal(checks, &report);
            Some(report)
        }
        Err(err) => {
            checks.push(fail_check(
                63,
                "benchmark_harness",
                "tool comparison template renders one governed row per benchmark stage",
                Some(DEFAULT_TOOL_COMPARISON_TEMPLATE_PATH.to_string()),
                format!("{err:#}"),
            ));
            None
        }
    }
}
