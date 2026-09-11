use super::declarations::BAM_GENOTYPING_COMPLETE_SCHEMA_VERSION;
use super::declarations::CHECKED_SURFACE_COUNT;
use super::declarations::EXPECTED_BAM_PIPELINE_ID;
use super::declarations::EXPECTED_CROSS_PIPELINE_ID;
use super::declarations::REQUIRED_OUTPUT_IDS;
use super::declarations::REQUIRED_PLAN_OUTPUT_IDS;
use super::models::BamGenotypingCompleteReport;
use super::models::BamGenotypingCompleteRow;
use super::paths::path_relative_to_repo;
use std::path::Path;
pub(super) fn project_report(
    repo_root: &Path,
    output_path: &Path,
    report_rows: Vec<BamGenotypingCompleteRow>,
    expected_tool_ids: Vec<String>,
) -> BamGenotypingCompleteReport {
    let complete_row_count =
        report_rows.iter().filter(|row| row.coverage_status == "complete").count();
    let incomplete_row_count = report_rows.len().saturating_sub(complete_row_count);
    let violations = report_rows
        .iter()
        .filter(|row| row.coverage_status != "complete")
        .cloned()
        .collect::<Vec<_>>();

    BamGenotypingCompleteReport {
        schema_version: BAM_GENOTYPING_COMPLETE_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, output_path),
        active_row_count: report_rows.len(),
        complete_row_count,
        incomplete_row_count,
        checked_surface_count: CHECKED_SURFACE_COUNT,
        expected_tool_ids,
        required_output_ids: REQUIRED_OUTPUT_IDS.iter().map(|value| (*value).to_string()).collect(),
        required_plan_output_ids: REQUIRED_PLAN_OUTPUT_IDS
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        bam_pipeline_id: EXPECTED_BAM_PIPELINE_ID.to_string(),
        cross_pipeline_id: EXPECTED_CROSS_PIPELINE_ID.to_string(),
        toolset_ready: violations.is_empty(),
        violation_count: violations.len(),
        ok: violations.is_empty(),
        rows: report_rows,
        violations,
    }
}
