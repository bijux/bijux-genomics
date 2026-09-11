use super::super::evidence::DryRunEvidence;
use super::super::DryRunInputPaths;
use super::diagnostics::{blocked_by, error_detail, fail_check, ok_check};
use bijux_dna_bench_model::BenchmarkLocalHpcDryRunReadinessGoalCheck as LocalHpcDryRunReadyGoalCheck;
use std::collections::BTreeSet;
pub(in crate::dry_run) fn asset_coverage(
    evidence: &DryRunEvidence,
    paths: &DryRunInputPaths<'_>,
) -> LocalHpcDryRunReadyGoalCheck {
    let asset_report = evidence.asset.as_ref().ok();
    let stage_benchmark_report = evidence.stage_benchmark.as_ref().ok();
    let candidate_report = evidence.candidate.as_ref().ok();
    let asset_error = evidence.asset.as_ref().err();
    let stage_benchmark_error = evidence.stage_benchmark.as_ref().err();
    let candidate_error = evidence.candidate.as_ref().err();

    let output_path = Some(
        asset_report.map_or_else(|| paths.asset.to_string(), |report| report.output_path.clone()),
    );
    let Some(asset_report) = asset_report else {
        return fail_check(
            481,
            "HPC asset staging dry-run",
            output_path,
            error_detail(asset_error),
        );
    };
    let Some(stage_benchmark_report) = stage_benchmark_report else {
        return fail_check(
            481,
            "HPC asset staging dry-run",
            output_path,
            blocked_by("stage benchmark array", stage_benchmark_error),
        );
    };
    let Some(candidate_report) = candidate_report else {
        return fail_check(
            481,
            "HPC asset staging dry-run",
            output_path,
            blocked_by("first HPC candidate manifest", candidate_error),
        );
    };
    let asset_result_ids =
        asset_report.jobs.iter().map(|job| job.result_id.as_str()).collect::<BTreeSet<_>>();
    let missing_candidate_results = candidate_report
        .rows
        .iter()
        .filter_map(|row| {
            (!asset_result_ids.contains(row.result_id.as_str())).then_some(row.result_id.as_str())
        })
        .collect::<Vec<_>>();
    if asset_report.selected_job_count != stage_benchmark_report.benchmark_job_count {
        return fail_check(
            481,
            "HPC asset staging dry-run",
            output_path,
            format!(
                "asset staging manifest covers {} benchmark jobs but stage benchmark array covers {}",
                asset_report.selected_job_count, stage_benchmark_report.benchmark_job_count
            ),
        );
    }
    if !missing_candidate_results.is_empty() {
        return fail_check(
            481,
            "HPC asset staging dry-run",
            output_path,
            format!(
                "asset staging manifest is missing candidate benchmark results: {}",
                missing_candidate_results.join(", ")
            ),
        );
    }
    ok_check(
        481,
        "HPC asset staging dry-run",
        output_path,
        format!(
            "validated {} benchmark jobs with {} staged inputs and candidate coverage for all {} first-run rows",
            asset_report.selected_job_count,
            asset_report.staged_input_count,
            candidate_report.selected_job_count
        ),
    )
}
