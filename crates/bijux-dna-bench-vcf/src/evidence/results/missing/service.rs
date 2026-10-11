use super::model::{VcfMissingResultReport, VcfMissingResultStatus};
use super::paths::{path_relative_to_repo, repo_relative_path};
use super::{
    contract::ensure_vcf_missing_result_report_contract,
    fixture::materialize_vcf_missing_result_fixture,
    projection::collect_vcf_missing_result_report_rows,
};
use super::{
    DEFAULT_VCF_MISSING_RESULT_REPORT_FIXTURE_ROOT, VCF_MISSING_RESULT_REPORT_SCHEMA_VERSION,
};
use anyhow::Result;
use bijux_dna_bench_vcf_evidence::model::expectations::VcfExpectedBenchmarkResultRow;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
/// Exercise missing-result reporting using admitted synthetic stage-result fixtures.
///
/// # Errors
/// Rejects invalid expected bindings, unsafe fixture paths, malformed results and publication errors.
pub fn render_vcf_missing_result_report(
    inputs: &crate::inputs::VcfAdapterInputs,
    expected_rows: &[VcfExpectedBenchmarkResultRow],
    repo_root: &Path,
    output_path: &Path,
) -> Result<VcfMissingResultReport> {
    let output_path = repo_relative_path(repo_root, output_path);
    let fake_result_root =
        repo_root.join(PathBuf::from(DEFAULT_VCF_MISSING_RESULT_REPORT_FIXTURE_ROOT));
    super::admission::admit_expected_results(inputs, expected_rows, repo_root, &fake_result_root)?;
    let fixture = materialize_vcf_missing_result_fixture(&fake_result_root, expected_rows)?;
    let rows = collect_vcf_missing_result_report_rows(
        repo_root,
        &fixture.fake_result_root,
        expected_rows,
    )?;

    let expected_row_count = rows.len();
    let present_result_row_count =
        rows.iter().filter(|row| row.result_status == VcfMissingResultStatus::Present).count();
    let missing_result_row_count = rows
        .iter()
        .filter(|row| row.result_status == VcfMissingResultStatus::MissingResult)
        .count();
    let mut report_section_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *report_section_counts.entry(row.report_section.clone()).or_default() += 1;
    }

    let report = VcfMissingResultReport {
        schema_version: VCF_MISSING_RESULT_REPORT_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        fake_result_root: path_relative_to_repo(repo_root, &fixture.fake_result_root),
        expected_row_count,
        present_result_row_count,
        missing_result_row_count,
        passes_behavior_test: false,
        removed_result_row_id: fixture.removed_result_row_id.clone(),
        removed_manifest_path: path_relative_to_repo(repo_root, &fixture.removed_manifest_path),
        report_section_counts,
        rows,
    };
    let report = ensure_vcf_missing_result_report_contract(report)?;
    bijux_dna_infra::atomic_write_json(&output_path, &report)?;
    Ok(report)
}
