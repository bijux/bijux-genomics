use crate::readiness::assessment::fail_check;
use crate::readiness::context::ReadinessContext;
use crate::readiness::declarations::DEFAULT_STAGE_OUTPUT_COMPLETION_REPORT_PATH;
use bijux_dna_bench_model::evaluate_benchmark_local_hpc_harness_readiness;
use bijux_dna_bench_model::BenchmarkLocalHpcHarnessObservation;
use bijux_dna_bench_model::BenchmarkLocalHpcSubmissionReadinessGoalCheck as BenchLocalHpcSubmissionReadyGoalCheck;
use bijux_dna_bench_model::BenchmarkLocalStageOutputCompletionReport as BenchLocalStageOutputCompletionReport;
use std::path::PathBuf;

pub(super) fn observe_outputs(
    context: &ReadinessContext<'_>,
    checks: &mut Vec<BenchLocalHpcSubmissionReadyGoalCheck>,
) -> Option<BenchLocalStageOutputCompletionReport> {
    let repo_root = context.repo_root;
    match context.backend.check_local_stage_output_completion(
        repo_root,
        PathBuf::from(context.paths.fake_run_root),
        PathBuf::from(DEFAULT_STAGE_OUTPUT_COMPLETION_REPORT_PATH),
    ) {
        Ok(report) => {
            checks.push(evaluate_benchmark_local_hpc_harness_readiness(
                BenchmarkLocalHpcHarnessObservation::OutputCompletion(&report),
            ));
            Some(report)
        }
        Err(err) => {
            checks.push(fail_check(
                58,
                "benchmark_harness",
                "output completion requires every declared stage output",
                Some(DEFAULT_STAGE_OUTPUT_COMPLETION_REPORT_PATH.to_string()),
                format!("{err:#}"),
            ));
            None
        }
    }
}
