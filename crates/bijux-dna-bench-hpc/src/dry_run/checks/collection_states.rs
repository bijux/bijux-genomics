use super::super::evidence::DryRunEvidence;
use super::super::DryRunInputPaths;
use super::diagnostics::{error_detail, fail_check, ok_check};
use bijux_dna_bench_model::BenchmarkLocalHpcDryRunReadinessGoalCheck as LocalHpcDryRunReadyGoalCheck;
pub(in crate::dry_run) fn collection_states(
    evidence: &DryRunEvidence,
    paths: &DryRunInputPaths<'_>,
) -> LocalHpcDryRunReadyGoalCheck {
    let result_collection_report = evidence.result_collection.as_ref().ok();
    let result_collection_error = evidence.result_collection.as_ref().err();

    let output_path =
        Some(result_collection_report.map_or_else(
            || paths.result_collection.to_string(),
            |report| report.output_path.clone(),
        ));
    let Some(result_collection_report) = result_collection_report else {
        return fail_check(
            488,
            "HPC result collection simulator",
            output_path,
            error_detail(result_collection_error),
        );
    };
    if !result_collection_report.behavior.proven {
        return fail_check(
            488,
            "HPC result collection simulator",
            output_path,
            "result-collection simulation did not prove distinct completion states".to_string(),
        );
    }
    let required_status_counts = [
        result_collection_report.complete_row_count,
        result_collection_report.failed_row_count,
        result_collection_report.missing_row_count,
        result_collection_report.insufficient_row_count,
        result_collection_report.unavailable_row_count,
    ];
    if required_status_counts.into_iter().any(|count| count == 0) {
        return fail_check(
            488,
            "HPC result collection simulator",
            output_path,
            "result-collection simulation no longer covers all five required collection states"
                .to_string(),
        );
    }
    ok_check(
        488,
        "HPC result collection simulator",
        output_path,
        format!(
            "validated {} collection rows spanning complete/failed/missing/insufficient/unavailable cases",
            result_collection_report.row_count
        ),
    )
}
