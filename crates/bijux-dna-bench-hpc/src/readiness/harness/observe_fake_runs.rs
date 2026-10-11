use crate::readiness::assessment::fail_check;
use crate::readiness::context::ReadinessContext;
use crate::readiness::harness::runs::evaluate_fake_run_success_goal;
use bijux_dna_bench_model::BenchmarkLocalHpcSubmissionReadinessGoalCheck as BenchLocalHpcSubmissionReadyGoalCheck;
use bijux_dna_bench_model::BenchmarkLocalStageFakeRunManifest as BenchLocalStageFakeRunManifest;
use std::path::PathBuf;

pub(super) fn observe_fake_runs(
    context: &ReadinessContext<'_>,
    checks: &mut Vec<BenchLocalHpcSubmissionReadyGoalCheck>,
) -> Option<BenchLocalStageFakeRunManifest> {
    let repo_root = context.repo_root;
    match context
        .backend
        .fake_run_local_stage_commands(repo_root, PathBuf::from(context.paths.fake_run_root))
    {
        Ok(manifest) => {
            evaluate_fake_run_success_goal(context, checks, &manifest);
            Some(manifest)
        }
        Err(err) => {
            checks.push(fail_check(
                56,
                "benchmark_harness",
                "fake-run stage harness materializes every governed stage output and manifest",
                Some(context.paths.fake_run_root.to_string()),
                format!("{err:#}"),
            ));
            None
        }
    }
}
