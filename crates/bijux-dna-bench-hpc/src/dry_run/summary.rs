use super::evidence::DryRunEvidence;
use bijux_dna_bench_model::BenchmarkLocalHpcDryRunReadinessSummary as LocalHpcDryRunReadySummary;
pub(in crate::dry_run) fn summary(evidence: &DryRunEvidence) -> LocalHpcDryRunReadySummary {
    let asset_report = evidence.asset.as_ref().ok();
    let scratch_report = evidence.scratch.as_ref().ok();
    let execution_report = evidence.execution.as_ref().ok();
    let stage_benchmark_report = evidence.stage_benchmark.as_ref().ok();
    let pipeline_node_report = evidence.pipeline_node.as_ref().ok();
    let dependency_report = evidence.dependency.as_ref().ok();
    let resume_report = evidence.resume.as_ref().ok();
    let result_collection_report = evidence.result_collection.as_ref().ok();
    let candidate_report = evidence.candidate.as_ref().ok();

    LocalHpcDryRunReadySummary {
        benchmark_job_count: stage_benchmark_report
            .map(|report| report.benchmark_job_count)
            .or_else(|| asset_report.map(|report| report.selected_job_count))
            .unwrap_or(0),
        essential_pipeline_job_count: pipeline_node_report
            .map(|report| report.pipeline_job_count)
            .or_else(|| scratch_report.map(|report| report.essential_pipeline_job_count))
            .unwrap_or(0),
        staged_input_count: asset_report.map_or(0, |report| report.staged_input_count),
        execution_resolver_row_count: execution_report.map_or(0, |report| report.row_count),
        execution_unclassified_tool_count: execution_report
            .map_or(0, |report| report.unclassified_tool_count),
        stage_benchmark_row_count: stage_benchmark_report.map_or(0, |report| report.rows.len()),
        pipeline_node_row_count: pipeline_node_report.map_or(0, |report| report.rows.len()),
        dependency_case_count: dependency_report.map_or(0, |report| report.case_count),
        resume_rerun_job_count: resume_report.map_or(0, |report| report.rerun_job_count),
        result_collection_row_count: result_collection_report.map_or(0, |report| report.row_count),
        candidate_job_count: candidate_report.map_or(0, |report| report.selected_job_count),
        candidate_representative_group_count: candidate_report
            .map_or(0, |report| report.representative_group_count),
        candidate_selected_input_bytes: candidate_report
            .map_or(0, |report| report.selected_staged_input_bytes),
    }
}
