use super::super::state::ReadinessState;
use crate::adapters::observations::VcfAdapterObservations;
use crate::evidence::expectations::collect_vcf_expected_benchmark_result_rows;
use crate::evidence::parsers::failure::{
    render_vcf_parser_failure_audit, DEFAULT_VCF_PARSER_FAILURE_TESTS_PATH,
};
use crate::evidence::results::missing::{
    render_vcf_missing_result_report, DEFAULT_VCF_MISSING_RESULT_REPORT_TEST_PATH,
};
use anyhow::{bail, Result};
use std::path::Path;
pub(super) fn parser_failures(repo_root: &Path) -> Result<String> {
    let report = render_vcf_parser_failure_audit(
        repo_root,
        Path::new(DEFAULT_VCF_PARSER_FAILURE_TESTS_PATH),
    )?;
    if report.row_count != 7 || report.passed_row_count != 7 || report.failed_row_count != 0 {
        bail!("VCF parser failure tests drifted from the governed case set");
    }
    Ok("validated 7 governed malformed-output VCF parser failure cases".to_string())
}

pub(super) fn missing_results(
    observations: &VcfAdapterObservations<'_>,
    state: &ReadinessState,
) -> Result<String> {
    let repo_root = observations.repo_root();
    let expected = collect_vcf_expected_benchmark_result_rows(observations)?;
    let report = render_vcf_missing_result_report(
        observations.inputs(),
        &expected,
        repo_root,
        Path::new(DEFAULT_VCF_MISSING_RESULT_REPORT_TEST_PATH),
    )?;
    if report.expected_row_count != state.counts.expected_results
        || report.present_result_row_count != state.counts.expected_results.saturating_sub(1)
        || report.missing_result_row_count != 1
        || !report.passes_behavior_test
    {
        bail!("VCF missing-result report drifted from the governed behavior contract");
    }
    Ok("validated governed missing-result behavior with one retained missing_result row"
        .to_string())
}
