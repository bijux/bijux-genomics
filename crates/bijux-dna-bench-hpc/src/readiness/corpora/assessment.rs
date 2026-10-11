use crate::readiness::assessment::fail_check;
use crate::readiness::assessment::ok_check;
use bijux_dna_bench_model::BenchmarkLocalCorpusSkipReport as LocalCorpusSkipReport;
use bijux_dna_bench_model::BenchmarkLocalCorpusStageCompatibilityReport as LocalCorpusStageCompatibilityValidationReport;
use bijux_dna_bench_model::BenchmarkLocalHpcSubmissionReadinessGoalCheck as BenchLocalHpcSubmissionReadyGoalCheck;

pub(in crate::readiness) fn evaluate_corpus_stage_compatibility_goal(
    checks: &mut Vec<BenchLocalHpcSubmissionReadyGoalCheck>,
    report: &LocalCorpusStageCompatibilityValidationReport,
) {
    if report.valid && report.stage_count == 51 {
        checks.push(ok_check(
            75,
            "corpora",
            "corpus-stage compatibility matrix validates all 51 governed stages",
            Some(report.matrix_path.clone()),
            format!(
                "compatibility matrix validates {} stages across {} fixtures",
                report.stage_count, report.fixture_count
            ),
        ));
    } else {
        checks.push(fail_check(
            75,
            "corpora",
            "corpus-stage compatibility matrix validates all 51 governed stages",
            Some(report.matrix_path.clone()),
            format!(
                "valid={} stage_count={} fixture_count={}",
                report.valid, report.stage_count, report.fixture_count
            ),
        ));
    }
}

pub(in crate::readiness) fn evaluate_corpus_skip_report_goal(
    checks: &mut Vec<BenchLocalHpcSubmissionReadyGoalCheck>,
    report: &LocalCorpusSkipReport,
    compatibility_report: Option<&LocalCorpusStageCompatibilityValidationReport>,
) {
    let planner_only_count_matches = compatibility_report.as_ref().is_none_or(|compatibility| {
        compatibility.planner_only_stage_count == report.planner_only_stage_count
    });
    let skip_rows_are_complete = report.skips.iter().all(|skip| {
        !skip.stage_id.trim().is_empty()
            && !skip.corpus_id.trim().is_empty()
            && !skip.reason.trim().is_empty()
            && !skip.replacement_corpus_id.trim().is_empty()
    });
    if report.stage_count == 51 && planner_only_count_matches && skip_rows_are_complete {
        checks.push(ok_check(
            76,
            "corpora",
            "corpus skip report keeps incompatible fixtures and planner-only stages explicit",
            Some(report.output_path.clone()),
            format!(
                "skip report records {} fixture skips and {} planner-only stages",
                report.skip_count, report.planner_only_stage_count
            ),
        ));
    } else {
        checks.push(fail_check(
            76,
            "corpora",
            "corpus skip report keeps incompatible fixtures and planner-only stages explicit",
            Some(report.output_path.clone()),
            format!(
                "stage_count={} planner_only_count_matches={} skip_rows_are_complete={}",
                report.stage_count, planner_only_count_matches, skip_rows_are_complete
            ),
        ));
    }
}
