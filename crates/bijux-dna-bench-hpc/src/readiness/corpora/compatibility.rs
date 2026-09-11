use crate::readiness::assessment::fail_check;
use crate::readiness::context::ReadinessContext;
use crate::readiness::corpora::assessment::evaluate_corpus_skip_report_goal;
use crate::readiness::corpora::assessment::evaluate_corpus_stage_compatibility_goal;
use crate::readiness::declarations::DEFAULT_CORPUS_SKIP_REPORT_PATH;
use bijux_dna_bench_model::BenchmarkLocalHpcSubmissionReadinessGoalCheck as BenchLocalHpcSubmissionReadyGoalCheck;

pub(in crate::readiness) fn evaluate_compatibility_fixture_goals(
    context: &ReadinessContext<'_>,
    checks: &mut Vec<BenchLocalHpcSubmissionReadyGoalCheck>,
) {
    let repo_root = context.repo_root;
    let compatibility_report = match context.backend.validate_corpus_stage_compatibility_path(
        repo_root,
        &repo_root.join(context.paths.corpus_stage_compatibility),
    ) {
        Ok(report) => {
            evaluate_corpus_stage_compatibility_goal(checks, &report);
            Some(report)
        }
        Err(err) => {
            checks.push(fail_check(
                75,
                "corpora",
                "corpus-stage compatibility matrix validates all 51 governed stages",
                Some(context.paths.corpus_stage_compatibility.to_string()),
                format!("{err:#}"),
            ));
            None
        }
    };

    match context.backend.render_corpus_skip_report_path(
        repo_root,
        &repo_root.join(context.paths.corpus_stage_compatibility),
        &repo_root.join(DEFAULT_CORPUS_SKIP_REPORT_PATH),
    ) {
        Ok(report) => {
            evaluate_corpus_skip_report_goal(checks, &report, compatibility_report.as_ref());
        }
        Err(err) => checks.push(fail_check(
            76,
            "corpora",
            "corpus skip report keeps incompatible fixtures and planner-only stages explicit",
            Some(DEFAULT_CORPUS_SKIP_REPORT_PATH.to_string()),
            format!("{err:#}"),
        )),
    }
}
