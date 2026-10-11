use crate::readiness::assessment::fail_check;
use crate::readiness::assessment::ok_check;
use crate::readiness::context::ReadinessContext;
use bijux_dna_bench_model::BenchmarkLocalHpcSubmissionReadinessGoalCheck as BenchLocalHpcSubmissionReadyGoalCheck;
use bijux_dna_bench_model::BenchmarkLocalStageCommandManifest as BenchLocalStageCommandManifest;

pub(in crate::readiness) fn evaluate_stage_command_render_goals(
    context: &ReadinessContext<'_>,
    checks: &mut Vec<BenchLocalHpcSubmissionReadyGoalCheck>,
    manifest: &BenchLocalStageCommandManifest,
) {
    let repo_root = context.repo_root;
    let script_path = repo_root.join(&manifest.script_output_path);
    let manifest_path = repo_root.join(&manifest.manifest_output_path);
    if script_path.is_file() && manifest.command_count == 51 {
        checks.push(ok_check(
            54,
            "benchmark_harness",
            "local stage commands render into one governed shell script",
            Some(manifest.script_output_path.clone()),
            format!("rendered {} stage commands", manifest.command_count),
        ));
    } else {
        checks.push(fail_check(
            54,
            "benchmark_harness",
            "local stage commands render into one governed shell script",
            Some(manifest.script_output_path.clone()),
            format!(
                "expected script file and 51 commands but found script_exists={} command_count={}",
                script_path.is_file(),
                manifest.command_count
            ),
        ));
    }

    let command_rows_are_complete = manifest.commands.iter().all(|entry| {
        !entry.stage_id.trim().is_empty()
            && !entry.tool_id.trim().is_empty()
            && !entry.command.trim().is_empty()
            && entry.threads > 0
            && entry.memory_mb > 0
    });
    if manifest_path.is_file() && manifest.commands.len() == 51 && command_rows_are_complete {
        checks.push(ok_check(
            55,
            "benchmark_harness",
            "rendered stage command manifest carries declared fields for every stage",
            Some(manifest.manifest_output_path.clone()),
            "all 51 rendered stage command rows carry stage, tool, resource, and command fields"
                .to_string(),
        ));
    } else {
        checks.push(fail_check(
            55,
            "benchmark_harness",
            "rendered stage command manifest carries declared fields for every stage",
            Some(manifest.manifest_output_path.clone()),
            format!(
                "manifest_exists={} row_count={} complete_rows={command_rows_are_complete}",
                manifest_path.is_file(),
                manifest.commands.len()
            ),
        ));
    }
}
