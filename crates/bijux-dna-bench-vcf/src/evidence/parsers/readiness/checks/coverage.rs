use super::super::service::ActiveBindingProvider;
use super::super::state::ReadinessState;
use crate::adapters::observations::VcfAdapterObservations;
use crate::evidence::expectations::collect_vcf_expected_benchmark_result_rows;
use crate::evidence::parsers::{
    render_vcf_parser_fixture_coverage, DEFAULT_VCF_PARSER_FIXTURE_COVERAGE_PATH,
};
use crate::evidence::scope::collect_vcf_active_stage_tool_matrix_rows_from_expected;
use anyhow::{bail, Result};
use std::path::Path;
pub(super) fn parser_coverage(
    observations: &VcfAdapterObservations<'_>,
    active_bindings: ActiveBindingProvider,
    state: &mut ReadinessState,
) -> Result<String> {
    let repo_root = observations.repo_root();
    let active = active_bindings(repo_root)?;
    let expected = collect_vcf_expected_benchmark_result_rows(observations)?;
    let retained = collect_vcf_active_stage_tool_matrix_rows_from_expected(
        observations.inputs(),
        repo_root,
        &active,
        &expected,
    )?;
    let report = render_vcf_parser_fixture_coverage(
        observations.inputs(),
        repo_root,
        &retained,
        Path::new(DEFAULT_VCF_PARSER_FIXTURE_COVERAGE_PATH),
    )?;
    if report.stage_count != 20
        || report.row_count != report.covered_row_count
        || report.missing_row_count != 0
        || report.parser_fixture_coverage_percent.partial_cmp(&100.0)
            != Some(std::cmp::Ordering::Equal)
    {
        bail!("VCF parser fixture coverage drifted from the governed active slice");
    }
    state.counts.active_fixtures = report.covered_row_count;
    let row_count = report.row_count;
    state.parser_coverage_report = Some(report);
    Ok(format!("validated full parser-fixture coverage across {row_count} active VCF rows"))
}
