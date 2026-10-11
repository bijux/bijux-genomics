use super::super::evidence::DryRunEvidence;
use super::super::DryRunInputPaths;
use super::diagnostics::{blocked_by, error_detail, fail_check, ok_check};
use bijux_dna_bench_model::BenchmarkLocalHpcDryRunReadinessGoalCheck as LocalHpcDryRunReadyGoalCheck;
pub(in crate::dry_run) fn pipeline_array(
    evidence: &DryRunEvidence,
    paths: &DryRunInputPaths<'_>,
) -> LocalHpcDryRunReadyGoalCheck {
    let pipeline_node_report = evidence.pipeline_node.as_ref().ok();
    let scratch_report = evidence.scratch.as_ref().ok();
    let pipeline_node_error = evidence.pipeline_node.as_ref().err();
    let scratch_error = evidence.scratch.as_ref().err();

    let output_path = Some(
        pipeline_node_report
            .map_or_else(|| paths.pipeline_node.to_string(), |report| report.script_path.clone()),
    );
    let Some(pipeline_node_report) = pipeline_node_report else {
        return fail_check(
            485,
            "SLURM array script for pipeline nodes",
            output_path,
            error_detail(pipeline_node_error),
        );
    };
    let Some(scratch_report) = scratch_report else {
        return fail_check(
            485,
            "SLURM array script for pipeline nodes",
            output_path,
            blocked_by("scratch layout", scratch_error),
        );
    };
    if pipeline_node_report.pipeline_job_count != scratch_report.essential_pipeline_job_count {
        return fail_check(
            485,
            "SLURM array script for pipeline nodes",
            output_path,
            format!(
                "pipeline node array covers {} rows but scratch layout covers {} essential pipeline jobs",
                pipeline_node_report.pipeline_job_count, scratch_report.essential_pipeline_job_count
            ),
        );
    }
    if pipeline_node_report.dependency_count == 0 {
        return fail_check(
            485,
            "SLURM array script for pipeline nodes",
            output_path,
            "pipeline node array did not preserve any dependency entries".to_string(),
        );
    }
    ok_check(
        485,
        "SLURM array script for pipeline nodes",
        output_path,
        format!(
            "validated {} pipeline-node rows with {} explicit dependency edges",
            pipeline_node_report.pipeline_job_count, pipeline_node_report.dependency_count
        ),
    )
}
