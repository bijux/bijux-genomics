use crate::readiness::assessment::check_goal;
use crate::readiness::context::ReadinessContext;
use anyhow::anyhow;
use bijux_dna_bench_model::BenchmarkLocalHpcSubmissionReadinessGoalCheck as BenchLocalHpcSubmissionReadyGoalCheck;

pub(in crate::readiness) fn evaluate_bam_fixture_goals(
    context: &ReadinessContext<'_>,
    checks: &mut Vec<BenchLocalHpcSubmissionReadyGoalCheck>,
) {
    let repo_root = context.repo_root;
    let bam_report = check_goal(
        66,
        "corpora",
        "corpus-01 mini BAM fixture validates aligned sample contracts",
        Some(context.paths.bam_fixture),
        || {
            let report = context.backend.validate_bam_corpus_fixture_manifest_path(
                repo_root,
                &repo_root.join(context.paths.bam_fixture),
            )?;
            if report.sample_count == 2 && report.valid {
                Ok(format!(
                    "corpus-01 mini BAM fixture validates {} aligned samples",
                    report.sample_count
                ))
            } else {
                Err(anyhow!(
                    "expected 2 BAM samples and valid fixture but found sample_count={} valid={}",
                    report.sample_count,
                    report.valid
                ))
            }
        },
    );
    checks.push(bam_report);
}
