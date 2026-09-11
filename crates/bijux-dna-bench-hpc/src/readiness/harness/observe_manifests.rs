use crate::readiness::assessment::fail_check;
use crate::readiness::context::ReadinessContext;
use crate::readiness::declarations::DEFAULT_STAGE_MANIFEST_COMPLETION_REPORT_PATH;
use bijux_dna_bench_model::evaluate_benchmark_local_hpc_harness_readiness;
use bijux_dna_bench_model::BenchmarkLocalHpcHarnessObservation;
use bijux_dna_bench_model::BenchmarkLocalHpcSubmissionReadinessGoalCheck as BenchLocalHpcSubmissionReadyGoalCheck;
use bijux_dna_bench_model::BenchmarkLocalStageManifestCompletionReport as BenchLocalStageManifestCompletionReport;
use std::path::PathBuf;

pub(super) fn observe_manifests(
    context: &ReadinessContext<'_>,
    checks: &mut Vec<BenchLocalHpcSubmissionReadyGoalCheck>,
) -> Option<BenchLocalStageManifestCompletionReport> {
    let repo_root = context.repo_root;
    match context.backend.check_local_stage_manifest_completion(
        repo_root,
        PathBuf::from(context.paths.fake_run_root),
        PathBuf::from(DEFAULT_STAGE_MANIFEST_COMPLETION_REPORT_PATH),
    ) {
        Ok(report) => {
            checks.push(evaluate_benchmark_local_hpc_harness_readiness(
                BenchmarkLocalHpcHarnessObservation::ManifestCompletion(&report),
            ));
            Some(report)
        }
        Err(err) => {
            checks.push(fail_check(
                59,
                "benchmark_harness",
                "manifest completion requires every stage-result manifest",
                Some(DEFAULT_STAGE_MANIFEST_COMPLETION_REPORT_PATH.to_string()),
                format!("{err:#}"),
            ));
            None
        }
    }
}
