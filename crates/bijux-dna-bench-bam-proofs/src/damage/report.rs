use super::declarations::BAM_DAMAGE_COMPLETE_SCHEMA_VERSION;
use super::declarations::CHECKED_SURFACE_COUNT;
use super::declarations::REQUIRED_NORMALIZED_METRIC_NAMES;
use super::declarations::REQUIRED_OUTPUT_IDS;
use super::declarations::REQUIRED_PARSER_OUTPUT_FIELDS;
use super::declarations::REQUIRED_SUMMARY_METRIC_NAMES;
use super::declarations::REQUIRED_UNIFIED_METRIC_NAMES;
use super::models::BamDamageCompleteReport;
use super::models::BamDamageCompleteRow;
use super::paths::path_relative_to_repo;
use std::path::Path;
// Projection consumes admitted observations; independent flags retain diagnostics.
#[allow(clippy::struct_excessive_bools)]
pub(super) struct ReportProjection<'a> {
    pub output_path: &'a Path,
    pub repo_root: &'a Path,
    pub rows: Vec<BamDamageCompleteRow>,
    pub expected_tool_ids: Vec<String>,
    pub observed_tool_ids: Vec<String>,
    pub missing_tool_ids: Vec<String>,
    pub unexpected_tool_ids: Vec<String>,
    pub toolset_ready: bool,
    pub complete_row_count: usize,
    pub violations: Vec<BamDamageCompleteRow>,
}
pub(super) fn project(input: ReportProjection<'_>) -> BamDamageCompleteReport {
    let ReportProjection {
        output_path,
        repo_root,
        rows,
        expected_tool_ids,
        observed_tool_ids,
        missing_tool_ids,
        unexpected_tool_ids,
        toolset_ready,
        complete_row_count,
        violations,
    } = input;
    BamDamageCompleteReport {
        schema_version: BAM_DAMAGE_COMPLETE_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, output_path),
        active_row_count: rows.len(),
        complete_row_count,
        incomplete_row_count: rows.len().saturating_sub(complete_row_count),
        checked_surface_count: CHECKED_SURFACE_COUNT,
        expected_tool_ids,
        observed_tool_ids,
        missing_tool_ids,
        unexpected_tool_ids,
        required_output_ids: REQUIRED_OUTPUT_IDS.iter().map(|value| (*value).to_string()).collect(),
        required_summary_metric_names: REQUIRED_SUMMARY_METRIC_NAMES
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        required_unified_metric_names: REQUIRED_UNIFIED_METRIC_NAMES
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        required_parser_output_fields: REQUIRED_PARSER_OUTPUT_FIELDS
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        required_normalized_metric_names: REQUIRED_NORMALIZED_METRIC_NAMES
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        toolset_ready,
        violation_count: violations.len() + usize::from(!toolset_ready),
        ok: toolset_ready && violations.is_empty(),
        rows,
        violations,
    }
}
