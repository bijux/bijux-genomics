use super::declarations::BAM_HAPLOGROUPS_COMPLETE_SCHEMA_VERSION;
use super::declarations::CHECKED_SURFACE_COUNT;
use super::declarations::REQUIRED_OUTPUT_IDS;
use super::models::BamHaplogroupsCompleteReport;
use super::models::BamHaplogroupsCompleteRow;
use super::observations::LocalHaplogroupsSmokeReport;
use super::paths::path_relative_to_repo;
use std::path::Path;
// Projection consumes admitted observations; independent flags retain diagnostics.
#[allow(clippy::struct_excessive_bools)]
pub(super) struct ReportProjection<'a> {
    pub output_path: &'a Path,
    pub repo_root: &'a Path,
    pub smoke_report: LocalHaplogroupsSmokeReport,
    pub expected_tool_ids: Vec<String>,
    pub report_rows: Vec<BamHaplogroupsCompleteRow>,
    pub complete_row_count: usize,
    pub incomplete_row_count: usize,
    pub violations: Vec<BamHaplogroupsCompleteRow>,
}
pub(super) fn project(input: ReportProjection<'_>) -> BamHaplogroupsCompleteReport {
    let ReportProjection {
        output_path,
        repo_root,
        smoke_report,
        expected_tool_ids,
        report_rows,
        complete_row_count,
        incomplete_row_count,
        violations,
    } = input;
    BamHaplogroupsCompleteReport {
        schema_version: BAM_HAPLOGROUPS_COMPLETE_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, output_path),
        active_row_count: report_rows.len(),
        complete_row_count,
        incomplete_row_count,
        checked_surface_count: CHECKED_SURFACE_COUNT,
        expected_tool_ids,
        required_output_ids: REQUIRED_OUTPUT_IDS.iter().map(|value| (*value).to_string()).collect(),
        local_smoke_sample_id: smoke_report.sample_id,
        local_smoke_case_count: smoke_report.case_count,
        toolset_ready: violations.is_empty(),
        violation_count: violations.len(),
        ok: violations.is_empty(),
        rows: report_rows,
        violations,
    }
}
