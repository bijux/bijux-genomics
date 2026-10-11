use crate::readiness::assessment::fail_check;
use crate::readiness::context::ReadinessContext;
use crate::readiness::declarations::DEFAULT_RUNTIME_METRICS_REPORT_PATH;
use bijux_dna_bench_model::evaluate_benchmark_local_hpc_harness_readiness;
use bijux_dna_bench_model::BenchmarkLocalHpcHarnessObservation;
use bijux_dna_bench_model::BenchmarkLocalHpcSubmissionReadinessGoalCheck as BenchLocalHpcSubmissionReadyGoalCheck;
use bijux_dna_bench_model::BenchmarkLocalStageRuntimeMetricsReport as BenchLocalStageRuntimeMetricsReport;
use std::path::PathBuf;

pub(super) fn observe_runtime(
    context: &ReadinessContext<'_>,
    checks: &mut Vec<BenchLocalHpcSubmissionReadyGoalCheck>,
) -> Option<BenchLocalStageRuntimeMetricsReport> {
    let repo_root = context.repo_root;
    match context.backend.collect_local_stage_runtime_metrics(
        repo_root,
        PathBuf::from(context.paths.fake_run_root),
        PathBuf::from(DEFAULT_RUNTIME_METRICS_REPORT_PATH),
    ) {
        Ok(report) => {
            checks.push(evaluate_benchmark_local_hpc_harness_readiness(
                BenchmarkLocalHpcHarnessObservation::RuntimeMetrics(&report),
            ));
            Some(report)
        }
        Err(err) => {
            checks.push(fail_check(
                61,
                "benchmark_harness",
                "runtime metrics extract validated start, finish, elapsed, exit, and status fields",
                Some(DEFAULT_RUNTIME_METRICS_REPORT_PATH.to_string()),
                format!("{err:#}"),
            ));
            None
        }
    }
}
