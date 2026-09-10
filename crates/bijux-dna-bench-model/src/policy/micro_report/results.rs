//! Owner: status-preserving report result projections.
use crate::{
    MicroBenchmarkCompleteRow, MicroBenchmarkExecutionStatus, MicroBenchmarkResultRow,
    MicroBenchmarkUnavailableRow,
};
pub(super) fn result_sections(
    result_rows: &[MicroBenchmarkResultRow],
) -> (Vec<MicroBenchmarkCompleteRow>, Vec<MicroBenchmarkUnavailableRow>) {
    let complete_rows = result_rows
        .iter()
        .filter(|row| row.status == MicroBenchmarkExecutionStatus::Succeeded)
        .map(|row| MicroBenchmarkCompleteRow {
            execution_id: row.execution_id.clone(),
            component_id: row.component_id.clone(),
            domain: row.domain.clone(),
            stage_id: row.stage_id.clone(),
            tool_id: row.tool_id.clone(),
            source_report_path: row.source_report_path.clone(),
            evidence_path: row.evidence_path.clone(),
            command: row.command.clone(),
            reason: row.reason.clone(),
            normalized_metric_count: row.normalized_metric_count,
            output_count: row.output_count,
            log_count: row.log_count,
        })
        .collect::<Vec<_>>();

    let unavailable_rows = result_rows
        .iter()
        .filter(|row| row.status != MicroBenchmarkExecutionStatus::Succeeded)
        .map(|row| MicroBenchmarkUnavailableRow {
            execution_id: row.execution_id.clone(),
            component_id: row.component_id.clone(),
            domain: row.domain.clone(),
            stage_id: row.stage_id.clone(),
            tool_id: row.tool_id.clone(),
            execution_status: row.status,
            source_report_path: row.source_report_path.clone(),
            command: row.command.clone(),
            reason: row.reason.clone(),
        })
        .collect::<Vec<_>>();

    (complete_rows, unavailable_rows)
}
