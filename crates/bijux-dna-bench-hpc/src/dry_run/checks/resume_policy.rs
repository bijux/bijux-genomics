use super::super::evidence::DryRunEvidence;
use super::super::DryRunInputPaths;
use super::diagnostics::{blocked_by, error_detail, fail_check, ok_check};
use bijux_dna_bench_model::BenchmarkLocalHpcDryRunReadinessGoalCheck as LocalHpcDryRunReadyGoalCheck;
pub(in crate::dry_run) fn resume_policy(
    evidence: &DryRunEvidence,
    paths: &DryRunInputPaths<'_>,
) -> LocalHpcDryRunReadyGoalCheck {
    let resume_report = evidence.resume.as_ref().ok();
    let scratch_report = evidence.scratch.as_ref().ok();
    let resume_error = evidence.resume.as_ref().err();
    let scratch_error = evidence.scratch.as_ref().err();

    let output_path = Some(
        resume_report.map_or_else(|| paths.resume.to_string(), |report| report.output_path.clone()),
    );
    let Some(resume_report) = resume_report else {
        return fail_check(487, "HPC resume simulator", output_path, error_detail(resume_error));
    };
    let Some(scratch_report) = scratch_report else {
        return fail_check(
            487,
            "HPC resume simulator",
            output_path,
            blocked_by("scratch layout", scratch_error),
        );
    };
    if !resume_report.behavior.proven {
        return fail_check(
            487,
            "HPC resume simulator",
            output_path,
            "resume simulation did not prove skip/rerun behavior".to_string(),
        );
    }
    if resume_report.job_count != scratch_report.selected_job_count {
        return fail_check(
            487,
            "HPC resume simulator",
            output_path,
            format!(
                "resume simulation covers {} jobs but scratch layout covers {}",
                resume_report.job_count, scratch_report.selected_job_count
            ),
        );
    }
    ok_check(
        487,
        "HPC resume simulator",
        output_path,
        format!(
            "validated explicit resume behavior across {} jobs with {} reruns and {} skips",
            resume_report.job_count, resume_report.rerun_job_count, resume_report.skip_job_count
        ),
    )
}
