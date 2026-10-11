use crate::readiness::assessment::fail_check;
use crate::readiness::context::ReadinessContext;
use crate::readiness::declarations::DEFAULT_STAGE_COMMANDS_PATH;
use crate::readiness::harness::commands::evaluate_stage_command_render_goals;
use bijux_dna_bench_model::BenchmarkLocalHpcSubmissionReadinessGoalCheck as BenchLocalHpcSubmissionReadyGoalCheck;
use bijux_dna_bench_model::BenchmarkLocalStageCommandManifest as BenchLocalStageCommandManifest;
use std::path::PathBuf;

pub(super) fn render_commands(
    context: &ReadinessContext<'_>,
    checks: &mut Vec<BenchLocalHpcSubmissionReadyGoalCheck>,
) -> Option<BenchLocalStageCommandManifest> {
    let repo_root = context.repo_root;
    match context
        .backend
        .render_local_stage_commands(repo_root, PathBuf::from(DEFAULT_STAGE_COMMANDS_PATH))
    {
        Ok(manifest) => {
            evaluate_stage_command_render_goals(context, checks, &manifest);
            Some(manifest)
        }
        Err(err) => {
            checks.push(fail_check(
                54,
                "benchmark_harness",
                "local stage commands render into one governed shell script",
                Some(DEFAULT_STAGE_COMMANDS_PATH.to_string()),
                format!("{err:#}"),
            ));
            checks.push(fail_check(
                55,
                "benchmark_harness",
                "rendered stage command manifest carries declared fields for every stage",
                Some(
                    "artifacts/benchmark-smoke/readiness/local-ready/rendered-stage-commands.json"
                        .to_string(),
                ),
                "stage command rendering failed earlier".to_string(),
            ));
            None
        }
    }
}
