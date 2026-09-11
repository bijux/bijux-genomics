use std::path::PathBuf;

use bijux_dna_bench_vcf::evidence::results::missing::{
    render_vcf_missing_result_report, VcfMissingResultStatus,
    DEFAULT_VCF_MISSING_RESULT_REPORT_TEST_PATH, VCF_MISSING_RESULT_REPORT_SCHEMA_VERSION,
};

fn repo_root() -> PathBuf {
    crate::support::repo_root()
}

#[test]
fn vcf_missing_result_report_tracks_removed_expected_result() {
    let root = repo_root();
    let inputs = crate::support::inputs(&root);
    let observations =
        bijux_dna_bench_vcf::adapters::observations::VcfAdapterObservations::new(&inputs, &root);
    let expected =
        bijux_dna_bench_vcf::evidence::expectations::collect_vcf_expected_benchmark_result_rows(
            &observations,
        )
        .expect("expected rows");
    let report = render_vcf_missing_result_report(
        &inputs,
        &expected,
        &root,
        &PathBuf::from(DEFAULT_VCF_MISSING_RESULT_REPORT_TEST_PATH),
    )
    .expect("render VCF missing-result report");

    assert_eq!(report.schema_version, VCF_MISSING_RESULT_REPORT_SCHEMA_VERSION);
    assert_eq!(report.output_path, DEFAULT_VCF_MISSING_RESULT_REPORT_TEST_PATH);
    assert_eq!(
        report.fake_result_root,
        "artifacts/benchmark-smoke/readiness/missing-result/vcf/results"
    );
    assert_eq!(report.expected_row_count, report.rows.len());
    assert_eq!(report.present_result_row_count, report.expected_row_count - 1);
    assert_eq!(report.missing_result_row_count, 1);
    assert!(report.passes_behavior_test);
    assert_eq!(report.report_section_counts.get("variant_calling").copied(), Some(6));
    assert_eq!(report.report_section_counts.get("imputation").copied(), Some(8));
    assert_eq!(report.report_section_counts.get("quality_control").copied(), Some(5));
    assert_eq!(report.report_section_counts.get("population_structure").copied(), Some(7));
    assert_eq!(report.report_section_counts.get("runs_of_homozygosity").copied(), Some(1));

    let removed_row = report
        .rows
        .iter()
        .find(|row| row.result_row_id == report.removed_result_row_id)
        .expect("removed result row");
    assert_eq!(removed_row.result_status, VcfMissingResultStatus::MissingResult);
    assert_eq!(removed_row.stage_id, "vcf.stats");
    assert_eq!(removed_row.tool_id, "bcftools");
    assert!(removed_row.observed_output_artifact_ids.is_empty());
    assert!(report.rows.iter().all(|row| {
        row.result_status == VcfMissingResultStatus::MissingResult
            || !row.observed_output_artifact_ids.is_empty()
    }));
}
