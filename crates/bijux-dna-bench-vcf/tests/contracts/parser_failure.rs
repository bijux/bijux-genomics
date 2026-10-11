use std::path::PathBuf;

use bijux_dna_bench_vcf::evidence::parsers::failure::{
    render_vcf_parser_failure_audit, DEFAULT_VCF_PARSER_FAILURE_TESTS_PATH,
    VCF_PARSER_FAILURE_TESTS_SCHEMA_VERSION,
};

fn repo_root() -> PathBuf {
    crate::support::repo_root()
}

#[test]
fn render_vcf_parser_failure_audit_reports_governed_case_set() {
    let root = repo_root();
    let report = render_vcf_parser_failure_audit(
        &root,
        &PathBuf::from(DEFAULT_VCF_PARSER_FAILURE_TESTS_PATH),
    )
    .expect("render VCF parser failure tests");

    assert_eq!(report.schema_version, VCF_PARSER_FAILURE_TESTS_SCHEMA_VERSION);
    assert_eq!(report.output_path, DEFAULT_VCF_PARSER_FAILURE_TESTS_PATH);
    assert_eq!(report.row_count, 7);
    assert_eq!(report.passed_row_count, 7);
    assert_eq!(report.failed_row_count, 0);
    assert_eq!(report.failure_reason_counts.get("empty_output"), Some(&1));
    assert_eq!(report.failure_reason_counts.get("malformed_vcf"), Some(&1));
    assert_eq!(report.failure_reason_counts.get("missing_index"), Some(&1));
    assert_eq!(report.failure_reason_counts.get("missing_sample_column"), Some(&1));
    assert_eq!(report.failure_reason_counts.get("malformed_pca_table"), Some(&1));
    assert_eq!(report.failure_reason_counts.get("malformed_imputation_quality_file"), Some(&1));
    assert_eq!(report.failure_reason_counts.get("malformed_segment_file"), Some(&1));
    assert!(report.rows.iter().all(|row| row.passed));
    assert!(report.rows.iter().any(|row| {
        row.stage_id == "vcf.postprocess"
            && row.tool_id == "bcftools"
            && row.failure_reason == "missing_index"
            && row.observed_error.contains("required tabix index for postprocess output is missing")
    }));
    assert!(report.rows.iter().any(|row| {
        row.stage_id == "vcf.phasing"
            && row.tool_id == "shapeit5"
            && row.failure_reason == "missing_sample_column"
            && row.observed_error.contains("phased VCF row is missing sample columns")
    }));
}
