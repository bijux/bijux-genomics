use super::super::evidence::DryRunEvidence;
use super::super::DryRunInputPaths;
use super::diagnostics::{blocked_by, error_detail, fail_check, ok_check};
use bijux_dna_bench_model::BenchmarkLocalHpcDryRunReadinessGoalCheck as LocalHpcDryRunReadyGoalCheck;
use std::collections::BTreeSet;
pub(in crate::dry_run) fn stage_array(
    evidence: &DryRunEvidence,
    paths: &DryRunInputPaths<'_>,
) -> LocalHpcDryRunReadyGoalCheck {
    let stage_benchmark_report = evidence.stage_benchmark.as_ref().ok();
    let asset_report = evidence.asset.as_ref().ok();
    let candidate_report = evidence.candidate.as_ref().ok();
    let stage_benchmark_error = evidence.stage_benchmark.as_ref().err();
    let asset_error = evidence.asset.as_ref().err();
    let candidate_error = evidence.candidate.as_ref().err();

    let output_path = Some(
        stage_benchmark_report
            .map_or_else(|| paths.stage_benchmark.to_string(), |report| report.script_path.clone()),
    );
    let Some(stage_benchmark_report) = stage_benchmark_report else {
        return fail_check(
            484,
            "SLURM array script for stage benchmarks",
            output_path,
            error_detail(stage_benchmark_error),
        );
    };
    let Some(asset_report) = asset_report else {
        return fail_check(
            484,
            "SLURM array script for stage benchmarks",
            output_path,
            blocked_by("asset staging manifest", asset_error),
        );
    };
    let Some(candidate_report) = candidate_report else {
        return fail_check(
            484,
            "SLURM array script for stage benchmarks",
            output_path,
            blocked_by("first HPC candidate manifest", candidate_error),
        );
    };
    let stage_results = stage_benchmark_report
        .rows
        .iter()
        .map(|row| row.result_id.as_str())
        .collect::<BTreeSet<_>>();
    let missing_candidate_results = candidate_report
        .rows
        .iter()
        .filter_map(|row| {
            (!stage_results.contains(row.result_id.as_str())).then_some(row.result_id.as_str())
        })
        .collect::<Vec<_>>();
    if stage_benchmark_report.benchmark_job_count != asset_report.selected_job_count {
        return fail_check(
            484,
            "SLURM array script for stage benchmarks",
            output_path,
            format!(
                "stage benchmark array covers {} rows but asset staging manifest covers {} benchmark jobs",
                stage_benchmark_report.benchmark_job_count, asset_report.selected_job_count
            ),
        );
    }
    if !missing_candidate_results.is_empty() {
        return fail_check(
            484,
            "SLURM array script for stage benchmarks",
            output_path,
            format!(
                "stage benchmark array is missing candidate result IDs: {}",
                missing_candidate_results.join(", ")
            ),
        );
    }
    ok_check(
        484,
        "SLURM array script for stage benchmarks",
        output_path,
        format!(
            "validated contiguous array mapping for {} benchmark rows with governed manifest {}",
            stage_benchmark_report.benchmark_job_count, stage_benchmark_report.manifest_path
        ),
    )
}
