use std::path::PathBuf;

use bijux_dna_bench_vcf::evidence::inputs::{
    render_vcf_adapter_missing_input_audit, DEFAULT_VCF_ADAPTER_MISSING_INPUT_TESTS_PATH,
    VCF_ADAPTER_MISSING_INPUT_TESTS_SCHEMA_VERSION,
};

fn repo_root() -> PathBuf {
    crate::support::repo_root()
}

#[test]
fn vcf_adapter_missing_input_audit_track_required_roles() {
    let repo_root = repo_root();
    let report = render_vcf_adapter_missing_input_audit(
        &crate::support::inputs(&repo_root),
        &repo_root,
        &repo_root.join(DEFAULT_VCF_ADAPTER_MISSING_INPUT_TESTS_PATH),
    )
    .expect("render report");

    assert_eq!(report.schema_version, VCF_ADAPTER_MISSING_INPUT_TESTS_SCHEMA_VERSION);
    assert_eq!(report.row_count, 10);
    assert_eq!(report.passed_row_count, 10);
    assert_eq!(report.failed_row_count, 0);
    assert_eq!(report.adapter_row_count, 9);
    assert_eq!(report.support_row_count, 1);
    assert!(report.rows.iter().any(|row| {
        row.missing_input_role == "target_sites_bed"
            && row.contract_surface == "fixture_support"
            && row.passed
    }));
    assert!(report.rows.iter().any(|row| {
        row.missing_input_role == "vcf_index"
            && row.stage_id == "vcf.phasing"
            && row.tool_id == "shapeit5"
            && row.passed
    }));
}
