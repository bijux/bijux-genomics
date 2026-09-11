use super::super::evidence::DryRunEvidence;
use super::super::DryRunInputPaths;
use super::diagnostics::{blocked_by, error_detail, fail_check, ok_check};
use bijux_dna_bench_model::BenchmarkLocalHpcDryRunReadinessGoalCheck as LocalHpcDryRunReadyGoalCheck;
use bijux_dna_bench_model::{
    BenchmarkLocalHpcAssetStagingManifest, BenchmarkLocalHpcCandidateRunManifest,
    BenchmarkLocalHpcExecutionResolverReport, BenchmarkLocalHpcResultCollectionSimulationReport,
    BenchmarkLocalHpcStageBenchmarkArrayReport,
};
use std::collections::{BTreeMap, BTreeSet};
pub(in crate::dry_run) fn candidate_admission(
    evidence: &DryRunEvidence,
    paths: &DryRunInputPaths<'_>,
) -> LocalHpcDryRunReadyGoalCheck {
    let candidate_report = evidence.candidate.as_ref().ok();
    let asset_report = evidence.asset.as_ref().ok();
    let stage_benchmark_report = evidence.stage_benchmark.as_ref().ok();
    let execution_report = evidence.execution.as_ref().ok();
    let result_collection_report = evidence.result_collection.as_ref().ok();
    let candidate_error = evidence.candidate.as_ref().err();
    let asset_error = evidence.asset.as_ref().err();
    let stage_benchmark_error = evidence.stage_benchmark.as_ref().err();
    let execution_error = evidence.execution.as_ref().err();
    let result_collection_error = evidence.result_collection.as_ref().err();

    let output_path = Some(
        candidate_report
            .map_or_else(|| paths.candidate.to_string(), |report| report.output_path.clone()),
    );
    let Some(candidate_report) = candidate_report else {
        return fail_check(
            489,
            "First small HPC candidate manifest",
            output_path,
            error_detail(candidate_error),
        );
    };
    let Some(asset_report) = asset_report else {
        return fail_check(
            489,
            "First small HPC candidate manifest",
            output_path,
            blocked_by("asset staging manifest", asset_error),
        );
    };
    let Some(stage_benchmark_report) = stage_benchmark_report else {
        return fail_check(
            489,
            "First small HPC candidate manifest",
            output_path,
            blocked_by("stage benchmark array", stage_benchmark_error),
        );
    };
    let Some(execution_report) = execution_report else {
        return fail_check(
            489,
            "First small HPC candidate manifest",
            output_path,
            blocked_by("execution resolver", execution_error),
        );
    };
    let Some(result_collection_report) = result_collection_report else {
        return fail_check(
            489,
            "First small HPC candidate manifest",
            output_path,
            blocked_by("result-collection simulation", result_collection_error),
        );
    };
    assess_admitted(
        CandidateInputs {
            candidate: candidate_report,
            asset: asset_report,
            stage_benchmark: stage_benchmark_report,
            execution: execution_report,
            result_collection: result_collection_report,
        },
        output_path,
    )
}
#[derive(Clone, Copy)]
struct CandidateInputs<'a> {
    candidate: &'a BenchmarkLocalHpcCandidateRunManifest,
    asset: &'a BenchmarkLocalHpcAssetStagingManifest,
    stage_benchmark: &'a BenchmarkLocalHpcStageBenchmarkArrayReport,
    execution: &'a BenchmarkLocalHpcExecutionResolverReport,
    result_collection: &'a BenchmarkLocalHpcResultCollectionSimulationReport,
}
fn assess_admitted(
    inputs: CandidateInputs<'_>,
    output_path: Option<String>,
) -> LocalHpcDryRunReadyGoalCheck {
    let CandidateInputs {
        candidate: candidate_report,
        asset: asset_report,
        stage_benchmark: stage_benchmark_report,
        execution: execution_report,
        result_collection: result_collection_report,
    } = inputs;
    if !candidate_report.behavior.proven {
        return fail_check(
            489,
            "First small HPC candidate manifest",
            output_path,
            "candidate manifest no longer proves the small governed first-run contract".to_string(),
        );
    }
    let asset_result_ids =
        asset_report.jobs.iter().map(|job| job.result_id.as_str()).collect::<BTreeSet<_>>();
    let stage_result_ids = stage_benchmark_report
        .rows
        .iter()
        .map(|row| row.result_id.as_str())
        .collect::<BTreeSet<_>>();
    let resolver_by_tool = execution_report
        .rows
        .iter()
        .map(|row| (row.tool_id.as_str(), row))
        .collect::<BTreeMap<_, _>>();
    for row in &candidate_report.rows {
        if !asset_result_ids.contains(row.result_id.as_str()) {
            return fail_check(
                489,
                "First small HPC candidate manifest",
                output_path,
                format!(
                    "candidate result `{}` is missing from asset staging manifest",
                    row.result_id
                ),
            );
        }
        if !stage_result_ids.contains(row.result_id.as_str()) {
            return fail_check(
                489,
                "First small HPC candidate manifest",
                output_path,
                format!(
                    "candidate result `{}` is missing from stage benchmark array",
                    row.result_id
                ),
            );
        }
        let Some(execution_row) = resolver_by_tool.get(row.tool_id.as_str()) else {
            return fail_check(
                489,
                "First small HPC candidate manifest",
                output_path,
                format!("candidate tool `{}` is missing from execution resolver", row.tool_id),
            );
        };
        if execution_row.execution_mode == "unclassified"
            || execution_row.resolution_kind.contains("unavailable")
        {
            return fail_check(
                489,
                "First small HPC candidate manifest",
                output_path,
                format!(
                    "candidate tool `{}` remains ambiguous: execution_mode=`{}` resolution_kind=`{}`",
                    row.tool_id, execution_row.execution_mode, execution_row.resolution_kind
                ),
            );
        }
    }
    if !result_collection_report.behavior.proven {
        return fail_check(
            489,
            "First small HPC candidate manifest",
            output_path,
            "candidate stop conditions are no longer backed by a proven result-collection simulator"
                .to_string(),
        );
    }
    ok_check(
        489,
        "First small HPC candidate manifest",
        output_path,
        format!(
            "validated {} first-run rows across {} representative groups with {} staged input bytes",
            candidate_report.selected_job_count,
            candidate_report.representative_group_count,
            candidate_report.selected_staged_input_bytes
        ),
    )
}
