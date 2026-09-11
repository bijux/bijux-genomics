use crate::readiness::assessment::check_goal;
use crate::readiness::context::ReadinessContext;
use anyhow::anyhow;
use bijux_dna_bench_model::BenchmarkLocalHpcSubmissionReadinessGoalCheck as BenchLocalHpcSubmissionReadyGoalCheck;

pub(in crate::readiness) fn evaluate_taxonomy_fixture_goals(
    context: &ReadinessContext<'_>,
    checks: &mut Vec<BenchLocalHpcSubmissionReadyGoalCheck>,
) {
    let repo_root = context.repo_root;
    let taxonomy_report = check_goal(
        69,
        "corpora",
        "taxonomy-mini fixture validates lineage, index, and backend bundle ownership",
        Some(context.paths.taxonomy_fixture),
        || {
            let report = context.backend.validate_taxonomy_database_fixture_manifest_path(
                repo_root,
                &repo_root.join(context.paths.taxonomy_fixture),
            )?;
            if report.valid && report.taxa_count == 3 {
                Ok(format!(
                    "taxonomy-mini fixture validates {} taxa with {} source records",
                    report.taxa_count, report.source_record_count
                ))
            } else {
                Err(anyhow!(
                    "expected valid taxonomy fixture with 3 taxa but found taxa_count={} valid={}",
                    report.taxa_count,
                    report.valid
                ))
            }
        },
    );
    checks.push(taxonomy_report);
}
