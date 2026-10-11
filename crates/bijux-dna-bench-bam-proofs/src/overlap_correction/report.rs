use super::declarations::BAM_OVERLAP_CORRECTION_COMPLETE_SCHEMA_VERSION;
use super::declarations::CHECKED_SURFACE_COUNT;
use super::models::BamOverlapCorrectionCompleteReport;
use super::models::BamOverlapCorrectionCompleteRow;
use super::paths::path_relative_to_repo;
use std::path::Path;
// Projection consumes admitted observations; independent flags retain diagnostics.
#[allow(clippy::struct_excessive_bools)]
pub(super) struct ReportProjection<'a> {
    pub output_path: &'a Path,
    pub repo_root: &'a Path,
    pub complete_row_count: usize,
    pub rows: Vec<BamOverlapCorrectionCompleteRow>,
    pub violations: Vec<BamOverlapCorrectionCompleteRow>,
}
pub(super) fn project(input: ReportProjection<'_>) -> BamOverlapCorrectionCompleteReport {
    let ReportProjection { output_path, repo_root, complete_row_count, rows, violations } = input;
    BamOverlapCorrectionCompleteReport {
        schema_version: BAM_OVERLAP_CORRECTION_COMPLETE_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, output_path),
        active_row_count: rows.len(),
        complete_row_count,
        incomplete_row_count: rows.len().saturating_sub(complete_row_count),
        checked_surface_count: CHECKED_SURFACE_COUNT,
        required_output_ids: rows[0].required_output_ids.clone(),
        required_local_smoke_fields: rows[0].required_local_smoke_fields.clone(),
        required_normalized_metric_names: rows[0].required_normalized_metric_names.clone(),
        violation_count: violations.len(),
        ok: violations.is_empty(),
        rows,
        violations,
    }
}
