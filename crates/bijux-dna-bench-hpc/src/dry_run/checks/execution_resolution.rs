use super::super::evidence::DryRunEvidence;
use super::super::DryRunInputPaths;
use super::diagnostics::{blocked_by, error_detail, fail_check, ok_check};
use bijux_dna_bench_model::BenchmarkLocalHpcDryRunReadinessGoalCheck as LocalHpcDryRunReadyGoalCheck;
use std::collections::BTreeMap;
pub(in crate::dry_run) fn execution_resolution(
    evidence: &DryRunEvidence,
    paths: &DryRunInputPaths<'_>,
) -> LocalHpcDryRunReadyGoalCheck {
    let execution_report = evidence.execution.as_ref().ok();
    let candidate_report = evidence.candidate.as_ref().ok();
    let execution_error = evidence.execution.as_ref().err();
    let candidate_error = evidence.candidate.as_ref().err();

    let output_path = Some(
        execution_report
            .map_or_else(|| paths.execution.to_string(), |report| report.output_path.clone()),
    );
    let Some(execution_report) = execution_report else {
        return fail_check(
            483,
            "HPC execution resolver",
            output_path,
            error_detail(execution_error),
        );
    };
    let Some(candidate_report) = candidate_report else {
        return fail_check(
            483,
            "HPC execution resolver",
            output_path,
            blocked_by("first HPC candidate manifest", candidate_error),
        );
    };
    let execution_by_tool = execution_report
        .rows
        .iter()
        .map(|row| (row.tool_id.as_str(), row))
        .collect::<BTreeMap<_, _>>();
    for row in &candidate_report.rows {
        let Some(execution_row) = execution_by_tool.get(row.tool_id.as_str()) else {
            return fail_check(
                483,
                "HPC execution resolver",
                output_path,
                format!("execution resolver is missing candidate tool `{}`", row.tool_id),
            );
        };
        if execution_row.execution_mode == "unclassified"
            || execution_row.resolution_kind.contains("unavailable")
        {
            return fail_check(
                483,
                "HPC execution resolver",
                output_path,
                format!(
                    "candidate tool `{}` remains unresolved: execution_mode=`{}` resolution_kind=`{}`",
                    row.tool_id, execution_row.execution_mode, execution_row.resolution_kind
                ),
            );
        }
    }
    ok_check(
        483,
        "HPC execution resolver",
        output_path,
        format!(
            "validated {} selected-tool rows; first-run candidate resolves {} tools without ambiguity",
            execution_report.row_count, candidate_report.selected_job_count
        ),
    )
}
