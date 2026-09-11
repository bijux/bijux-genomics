use crate::readiness::assessment::check_goal;
use crate::readiness::context::ReadinessContext;
use anyhow::anyhow;
use bijux_dna_bench_model::BenchmarkLocalHpcSubmissionReadinessGoalCheck as BenchLocalHpcSubmissionReadyGoalCheck;

pub(in crate::readiness) fn evaluate_fastq_fixture_goals(
    context: &ReadinessContext<'_>,
    checks: &mut Vec<BenchLocalHpcSubmissionReadyGoalCheck>,
) {
    let repo_root = context.repo_root;
    let fastq_report = check_goal(
        65,
        "corpora",
        "corpus-01 mini FASTQ fixture validates sample identity and read counts",
        Some(context.paths.fastq_fixture),
        || {
            let report = context.backend.validate_fastq_corpus_fixture_manifest_path(
                repo_root,
                &repo_root.join(context.paths.fastq_fixture),
            )?;
            if report.sample_count == 4 && report.valid {
                Ok(format!(
                    "corpus-01 mini FASTQ fixture validates {} samples",
                    report.sample_count
                ))
            } else {
                Err(anyhow!(
                    "expected 4 FASTQ samples and valid fixture but found sample_count={} valid={}",
                    report.sample_count,
                    report.valid
                ))
            }
        },
    );
    checks.push(fastq_report);
}
