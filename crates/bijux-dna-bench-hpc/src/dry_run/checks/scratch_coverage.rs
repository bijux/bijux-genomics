use super::super::evidence::DryRunEvidence;
use super::super::DryRunInputPaths;
use super::diagnostics::{blocked_by, error_detail, fail_check, ok_check};
use bijux_dna_bench_model::BenchmarkLocalHpcDryRunReadinessGoalCheck as LocalHpcDryRunReadyGoalCheck;
use bijux_dna_bench_model::{
    BenchmarkLocalHpcDependencySimulationReport, BenchmarkLocalHpcPipelineNodeArrayReport,
    BenchmarkLocalHpcResumeSimulationReport, BenchmarkLocalHpcScratchLayout,
    BenchmarkLocalHpcStageBenchmarkArrayReport,
};
pub(in crate::dry_run) fn scratch_coverage(
    evidence: &DryRunEvidence,
    paths: &DryRunInputPaths<'_>,
) -> LocalHpcDryRunReadyGoalCheck {
    let scratch_report = evidence.scratch.as_ref().ok();
    let stage_benchmark_report = evidence.stage_benchmark.as_ref().ok();
    let pipeline_node_report = evidence.pipeline_node.as_ref().ok();
    let dependency_report = evidence.dependency.as_ref().ok();
    let resume_report = evidence.resume.as_ref().ok();
    let scratch_error = evidence.scratch.as_ref().err();
    let stage_benchmark_error = evidence.stage_benchmark.as_ref().err();
    let pipeline_node_error = evidence.pipeline_node.as_ref().err();
    let dependency_error = evidence.dependency.as_ref().err();
    let resume_error = evidence.resume.as_ref().err();

    let output_path = Some(
        scratch_report
            .map_or_else(|| paths.scratch.to_string(), |report| report.output_path.clone()),
    );
    let Some(scratch_report) = scratch_report else {
        return fail_check(
            482,
            "HPC scratch layout dry-run",
            output_path,
            error_detail(scratch_error),
        );
    };
    let Some(stage_benchmark_report) = stage_benchmark_report else {
        return fail_check(
            482,
            "HPC scratch layout dry-run",
            output_path,
            blocked_by("stage benchmark array", stage_benchmark_error),
        );
    };
    let Some(pipeline_node_report) = pipeline_node_report else {
        return fail_check(
            482,
            "HPC scratch layout dry-run",
            output_path,
            blocked_by("pipeline node array", pipeline_node_error),
        );
    };
    let Some(dependency_report) = dependency_report else {
        return fail_check(
            482,
            "HPC scratch layout dry-run",
            output_path,
            blocked_by("dependency simulation", dependency_error),
        );
    };
    let Some(resume_report) = resume_report else {
        return fail_check(
            482,
            "HPC scratch layout dry-run",
            output_path,
            blocked_by("resume simulation", resume_error),
        );
    };
    assess_admitted(
        ScratchInputs {
            scratch: scratch_report,
            stage_benchmark: stage_benchmark_report,
            pipeline_node: pipeline_node_report,
            dependency: dependency_report,
            resume: resume_report,
        },
        output_path,
    )
}
#[derive(Clone, Copy)]
struct ScratchInputs<'a> {
    scratch: &'a BenchmarkLocalHpcScratchLayout,
    stage_benchmark: &'a BenchmarkLocalHpcStageBenchmarkArrayReport,
    pipeline_node: &'a BenchmarkLocalHpcPipelineNodeArrayReport,
    dependency: &'a BenchmarkLocalHpcDependencySimulationReport,
    resume: &'a BenchmarkLocalHpcResumeSimulationReport,
}
fn assess_admitted(
    inputs: ScratchInputs<'_>,
    output_path: Option<String>,
) -> LocalHpcDryRunReadyGoalCheck {
    let ScratchInputs {
        scratch: scratch_report,
        stage_benchmark: stage_benchmark_report,
        pipeline_node: pipeline_node_report,
        dependency: dependency_report,
        resume: resume_report,
    } = inputs;
    if scratch_report.benchmark_job_count != stage_benchmark_report.benchmark_job_count {
        return fail_check(
            482,
            "HPC scratch layout dry-run",
            output_path,
            format!(
                "scratch layout covers {} benchmark jobs but stage benchmark array covers {}",
                scratch_report.benchmark_job_count, stage_benchmark_report.benchmark_job_count
            ),
        );
    }
    if scratch_report.essential_pipeline_job_count != pipeline_node_report.pipeline_job_count {
        return fail_check(
            482,
            "HPC scratch layout dry-run",
            output_path,
            format!(
                "scratch layout covers {} essential pipeline jobs but pipeline node array covers {}",
                scratch_report.essential_pipeline_job_count, pipeline_node_report.pipeline_job_count
            ),
        );
    }
    for (surface, count) in [
        ("dependency simulation", dependency_report.job_count),
        ("resume simulation", resume_report.job_count),
    ] {
        if scratch_report.selected_job_count != count {
            return fail_check(
                482,
                "HPC scratch layout dry-run",
                output_path,
                format!(
                    "scratch layout covers {} selected jobs but {} covers {}",
                    scratch_report.selected_job_count, surface, count
                ),
            );
        }
    }
    ok_check(
        482,
        "HPC scratch layout dry-run",
        output_path,
        format!(
            "validated {} total jobs with {} benchmark roots, {} pipeline roots, and {} staged input links",
            scratch_report.selected_job_count,
            scratch_report.benchmark_job_count,
            scratch_report.essential_pipeline_job_count,
            scratch_report.input_link_count
        ),
    )
}
