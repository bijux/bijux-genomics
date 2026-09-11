use super::contracts::required_tool_ids;
use super::contracts::required_validated_reads_manifest_fields;
use super::contracts::required_validation_report_fields;
use super::declarations::COVERAGE_STATUS_COMPLETE;
use super::declarations::FAILURE_REASON_FIELD_ID;
use super::declarations::FASTQ_VALIDATE_READS_READY_SCHEMA_VERSION;
use super::declarations::VALIDATION_STATUS_FIELD_ID;
use super::models::FastqValidateReadsReadyReport;
use super::paths::path_relative_to_repo;
use super::source_inputs::EvaluatedRows;
use std::collections::BTreeMap;
use std::path::Path;
pub(super) fn summarize(
    repo_root: &Path,
    output_path: &Path,
    evaluated: EvaluatedRows,
) -> FastqValidateReadsReadyReport {
    let EvaluatedRows { rows, smoke } = evaluated;
    let complete_row_count =
        rows.iter().filter(|row| row.coverage_status == COVERAGE_STATUS_COMPLETE).count();
    let incomplete_row_count = rows.len().saturating_sub(complete_row_count);
    let mut coverage_status_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *coverage_status_counts.entry(row.coverage_status.clone()).or_default() += 1;
    }
    let violations = rows
        .iter()
        .filter(|row| row.coverage_status != COVERAGE_STATUS_COMPLETE)
        .cloned()
        .collect::<Vec<_>>();

    FastqValidateReadsReadyReport {
        schema_version: FASTQ_VALIDATE_READS_READY_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, output_path),
        active_row_count: rows.len(),
        complete_row_count,
        incomplete_row_count,
        checked_surface_count: 8,
        expected_tool_ids: required_tool_ids(),
        validation_status_field_id: VALIDATION_STATUS_FIELD_ID,
        failure_reason_field_id: FAILURE_REASON_FIELD_ID,
        required_validation_report_fields: required_validation_report_fields(),
        required_validated_reads_manifest_fields: required_validated_reads_manifest_fields(),
        sample_case_count: smoke.sample_case_count,
        smoke_sample_ids: smoke.sample_ids.clone(),
        smoke_layouts: smoke.layouts.clone(),
        coverage_status_counts,
        violation_count: violations.len(),
        ok: violations.is_empty(),
        rows,
        violations,
    }
}
