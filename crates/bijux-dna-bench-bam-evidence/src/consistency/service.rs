use super::{
    assessment::build_surface_check,
    declarations::{BAM_ACTIVE_ROW_CONSISTENCY_SCHEMA_VERSION, CHECKED_SURFACE_COUNT},
    models::BamActiveRowConsistencyReport,
    observation::observe,
};
use crate::{
    output_contract::render_bam_adapter_output_contract,
    parser_coverage::render_bam_parser_fixture_coverage, publication::path_relative_to_repo,
    report_map::render_bam_report_map, EvidenceContext,
};
use anyhow::Result;
use std::collections::BTreeSet;
use std::path::Path;
pub(super) fn build_bam_active_row_consistency_report(
    context: &EvidenceContext<'_>,
    output_path: &Path,
) -> Result<BamActiveRowConsistencyReport> {
    let active = context.consistency.publish_active_bindings(context.repo_root)?;
    let commands = context.consistency.publish_commands(context.repo_root)?;
    let outputs =
        render_bam_adapter_output_contract(context, Path::new(context.paths.output_contract))?;
    let expected = context.consistency.publish_expected_results(context.repo_root)?;
    let parser =
        render_bam_parser_fixture_coverage(context, Path::new(context.paths.parser_coverage))?;
    let report = render_bam_report_map(context, Path::new(context.paths.report_map))?;
    let active_bindings = active.bindings.iter().cloned().collect::<BTreeSet<_>>();
    let output_surface = observe(
        outputs.output_path,
        outputs.row_count,
        outputs.rows.iter().map(|row| (row.stage_id.as_str(), row.tool_id.as_str())),
    );
    let mut parser_surface = observe(
        parser.output_path,
        parser.row_count,
        parser.rows.iter().map(|row| (row.stage_id.as_str(), row.tool_id.as_str())),
    );
    parser_surface.stage_count = parser.stage_count;
    parser_surface.tool_count = parser.tool_count;
    let mut report_surface = observe(
        report.output_path,
        report.row_count,
        report.rows.iter().map(|row| (row.stage_id.as_str(), row.tool_id.as_str())),
    );
    report_surface.stage_count = report.stage_count;
    report_surface.tool_count = report.tool_count;
    let mut local_jobs = commands.commands.clone();
    local_jobs.output_path = commands.argv_output_path;
    let surfaces = vec![
        build_surface_check(
            "bam_rendered_commands",
            &commands.commands,
            &active_bindings,
            "validated one BAM rendered command row per active BAM binding",
        ),
        build_surface_check(
            "bam_output_declarations",
            &output_surface,
            &active_bindings,
            "validated one BAM output-declaration row per active BAM binding",
        ),
        build_surface_check(
            "bam_expected_results",
            &expected,
            &active_bindings,
            "validated one BAM expected-result row per active BAM binding",
        ),
        build_surface_check(
            "bam_parser_fixtures",
            &parser_surface,
            &active_bindings,
            "validated one BAM parser-fixture row per active BAM binding",
        ),
        build_surface_check(
            "bam_local_jobs",
            &local_jobs,
            &active_bindings,
            "validated one BAM local-job row per active BAM binding",
        ),
        build_surface_check(
            "bam_report_map",
            &report_surface,
            &active_bindings,
            "validated one BAM report-map row per active BAM binding",
        ),
    ];
    let passed_surface_count = surfaces.iter().filter(|surface| surface.ok).count();
    let failed_surfaces =
        surfaces.iter().filter(|surface| !surface.ok).cloned().collect::<Vec<_>>();
    let failed_surface_count = failed_surfaces.len();
    Ok(BamActiveRowConsistencyReport {
        schema_version: BAM_ACTIVE_ROW_CONSISTENCY_SCHEMA_VERSION,
        output_path: path_relative_to_repo(context.repo_root, output_path),
        active_matrix_output_path: active.output_path,
        active_row_count: active.row_count,
        active_stage_count: active.stage_count,
        active_tool_count: active.tool_count,
        checked_surface_count: surfaces.len(),
        passed_surface_count,
        failed_surface_count,
        ok: failed_surface_count == 0
            && surfaces.len() == CHECKED_SURFACE_COUNT
            && active.row_count == active_bindings.len(),
        surfaces,
        failed_surfaces,
    })
}
