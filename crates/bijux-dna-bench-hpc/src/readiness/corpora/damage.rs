use crate::readiness::assessment::check_goal;
use crate::readiness::context::ReadinessContext;
use anyhow::anyhow;
use bijux_dna_bench_model::BenchmarkLocalHpcSubmissionReadinessGoalCheck as BenchLocalHpcSubmissionReadyGoalCheck;

pub(in crate::readiness) fn evaluate_damage_fixture_goals(
    context: &ReadinessContext<'_>,
    checks: &mut Vec<BenchLocalHpcSubmissionReadyGoalCheck>,
) {
    let repo_root = context.repo_root;
    let damage_report = check_goal(
        67,
        "corpora",
        "corpus-01 damage fixture validates expected aDNA limitations and evidence",
        Some(context.paths.damage_fixture),
        || {
            let report = context.backend.validate_bam_damage_fixture_manifest_path(
                repo_root,
                &repo_root.join(context.paths.damage_fixture),
            )?;
            if report.valid && !report.expected_damage_path.trim().is_empty() {
                Ok(format!(
                    "damage fixture validates expected evidence at {}",
                    report.expected_damage_path
                ))
            } else {
                Err(anyhow!("damage fixture did not produce a valid expected-damage contract"))
            }
        },
    );
    checks.push(damage_report);
}
