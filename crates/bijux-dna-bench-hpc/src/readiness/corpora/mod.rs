use crate::readiness::context::ReadinessContext;
use bijux_dna_bench_model::BenchmarkLocalHpcSubmissionReadinessGoalCheck as BenchLocalHpcSubmissionReadyGoalCheck;

mod amplicon;
mod assessment;
mod bam;
mod compatibility;
mod damage;
mod edna;
mod fastq;
mod taxonomy;
pub(in crate::readiness) fn evaluate_corpus_goals(
    context: &ReadinessContext<'_>,
    checks: &mut Vec<BenchLocalHpcSubmissionReadyGoalCheck>,
) {
    fastq::evaluate_fastq_fixture_goals(context, checks);
    bam::evaluate_bam_fixture_goals(context, checks);
    damage::evaluate_damage_fixture_goals(context, checks);
    edna::evaluate_edna_fixture_goals(context, checks);
    taxonomy::evaluate_taxonomy_fixture_goals(context, checks);
    amplicon::evaluate_amplicon_fixture_goals(context, checks);
    compatibility::evaluate_compatibility_fixture_goals(context, checks);
}
