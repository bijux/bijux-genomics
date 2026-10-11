use super::declarations::CHECKED_SURFACE_COUNT;
use super::declarations::COVERAGE_STATUS_COMPLETE;
use super::declarations::DUPLICATE_STAGE_SPECS;
use super::declarations::FASTQ_DUPLICATE_STAGES_READY_SCHEMA_VERSION;
use super::models::FastqDuplicateStagesReadyReport;
use super::models::FastqDuplicateStagesReadyRow;
use super::paths::path_relative_to_repo;
use std::collections::BTreeMap;
use std::path::Path;
pub(super) fn summarize(
    repo_root: &Path,
    output_path: &Path,
    rows: Vec<FastqDuplicateStagesReadyRow>,
) -> FastqDuplicateStagesReadyReport {
    let complete_row_count =
        rows.iter().filter(|row| row.coverage_status == COVERAGE_STATUS_COMPLETE).count();
    let incomplete_row_count = rows.len().saturating_sub(complete_row_count);
    let violations = rows
        .iter()
        .filter(|row| row.coverage_status != COVERAGE_STATUS_COMPLETE)
        .cloned()
        .collect::<Vec<_>>();
    let mut coverage_status_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *coverage_status_counts.entry(row.coverage_status.clone()).or_default() += 1;
    }

    FastqDuplicateStagesReadyReport {
        schema_version: FASTQ_DUPLICATE_STAGES_READY_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, output_path),
        active_row_count: rows.len(),
        complete_row_count,
        incomplete_row_count,
        checked_surface_count: CHECKED_SURFACE_COUNT,
        stage_count: DUPLICATE_STAGE_SPECS.len(),
        expected_tool_ids_by_stage: DUPLICATE_STAGE_SPECS
            .iter()
            .map(|spec| {
                (
                    spec.stage_id.to_string(),
                    spec.expected_tool_ids.iter().map(|tool_id| (*tool_id).to_string()).collect(),
                )
            })
            .collect(),
        required_output_ids_by_stage: DUPLICATE_STAGE_SPECS
            .iter()
            .map(|spec| {
                (
                    spec.stage_id.to_string(),
                    spec.required_output_ids
                        .iter()
                        .map(|output_id| (*output_id).to_string())
                        .collect(),
                )
            })
            .collect(),
        required_metric_fields_by_stage: DUPLICATE_STAGE_SPECS
            .iter()
            .map(|spec| {
                (
                    spec.stage_id.to_string(),
                    spec.required_metric_fields.iter().map(|field| (*field).to_string()).collect(),
                )
            })
            .collect(),
        report_targets_by_stage: DUPLICATE_STAGE_SPECS
            .iter()
            .map(|spec| {
                (
                    spec.stage_id.to_string(),
                    vec![
                        spec.required_report_section_id.to_string(),
                        spec.required_summary_table_id.to_string(),
                    ],
                )
            })
            .collect(),
        coverage_status_counts,
        violation_count: violations.len(),
        ok: violations.is_empty(),
        rows,
        violations,
    }
}
