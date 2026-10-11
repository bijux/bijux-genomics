use super::declarations::BAM_KINSHIP_COMPLETE_SCHEMA_VERSION;
use super::declarations::CHECKED_SURFACE_COUNT;
use super::declarations::EXPECTED_BAM_PIPELINE_ID;
use super::declarations::EXPECTED_VCF_PIPELINE_ID;
use super::declarations::REQUIRED_OUTPUT_IDS;
use super::models::BamKinshipCompleteReport;
use super::models::BamKinshipCompleteRow;
use super::paths::path_relative_to_repo;
use std::path::Path;
pub(super) fn project_report(
    repo_root: &Path,
    output_path: &Path,
    report_rows: Vec<BamKinshipCompleteRow>,
    expected_tool_ids: Vec<String>,
    local_smoke_case_count: usize,
) -> BamKinshipCompleteReport {
    let complete_row_count =
        report_rows.iter().filter(|row| row.coverage_status == "complete").count();
    let incomplete_row_count = report_rows.len().saturating_sub(complete_row_count);
    let violations = report_rows
        .iter()
        .filter(|row| row.coverage_status != "complete")
        .cloned()
        .collect::<Vec<_>>();

    BamKinshipCompleteReport {
        schema_version: BAM_KINSHIP_COMPLETE_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, output_path),
        active_row_count: report_rows.len(),
        complete_row_count,
        incomplete_row_count,
        checked_surface_count: CHECKED_SURFACE_COUNT,
        expected_tool_ids,
        required_output_ids: REQUIRED_OUTPUT_IDS.iter().map(|value| (*value).to_string()).collect(),
        local_smoke_case_count,
        bam_pipeline_id: EXPECTED_BAM_PIPELINE_ID.to_string(),
        vcf_pipeline_id: EXPECTED_VCF_PIPELINE_ID.to_string(),
        toolset_ready: violations.is_empty(),
        violation_count: violations.len(),
        ok: violations.is_empty(),
        rows: report_rows,
        violations,
    }
}
