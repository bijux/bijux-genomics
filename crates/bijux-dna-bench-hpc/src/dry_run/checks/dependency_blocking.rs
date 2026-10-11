use super::super::evidence::DryRunEvidence;
use super::super::DryRunInputPaths;
use super::diagnostics::{blocked_by, error_detail, fail_check, ok_check};
use bijux_dna_bench_model::BenchmarkLocalHpcDryRunReadinessGoalCheck as LocalHpcDryRunReadyGoalCheck;
pub(in crate::dry_run) fn dependency_blocking(
    evidence: &DryRunEvidence,
    paths: &DryRunInputPaths<'_>,
) -> LocalHpcDryRunReadyGoalCheck {
    let dependency_report = evidence.dependency.as_ref().ok();
    let scratch_report = evidence.scratch.as_ref().ok();
    let dependency_error = evidence.dependency.as_ref().err();
    let scratch_error = evidence.scratch.as_ref().err();

    let output_path = Some(
        dependency_report
            .map_or_else(|| paths.dependency.to_string(), |report| report.output_path.clone()),
    );
    let Some(dependency_report) = dependency_report else {
        return fail_check(
            486,
            "SLURM dependency simulator",
            output_path,
            error_detail(dependency_error),
        );
    };
    let Some(scratch_report) = scratch_report else {
        return fail_check(
            486,
            "SLURM dependency simulator",
            output_path,
            blocked_by("scratch layout", scratch_error),
        );
    };
    if !dependency_report.all_cases_proven {
        return fail_check(
            486,
            "SLURM dependency simulator",
            output_path,
            "dependency simulation did not prove descendant-only blocking".to_string(),
        );
    }
    if dependency_report.job_count != scratch_report.selected_job_count {
        return fail_check(
            486,
            "SLURM dependency simulator",
            output_path,
            format!(
                "dependency simulation covers {} jobs but scratch layout covers {}",
                dependency_report.job_count, scratch_report.selected_job_count
            ),
        );
    }
    ok_check(
        486,
        "SLURM dependency simulator",
        output_path,
        format!(
            "validated {} failure cases across {} governed HPC jobs",
            dependency_report.case_count, dependency_report.job_count
        ),
    )
}
