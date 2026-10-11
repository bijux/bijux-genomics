use std::path::PathBuf;

use bijux_dna_bench_vcf::adapters::imputation::{
    render_vcf_imputation_family_adapter, DEFAULT_VCF_IMPUTATION_FAMILY_ADAPTER_PATH,
};

fn repo_root() -> PathBuf {
    crate::support::repo_root()
}

#[test]
fn vcf_imputation_family_adapter_tracks_governed_rows() {
    let repo_root = repo_root();
    let report = render_vcf_imputation_family_adapter(
        &crate::support::inputs(&repo_root),
        &repo_root,
        &PathBuf::from(DEFAULT_VCF_IMPUTATION_FAMILY_ADAPTER_PATH),
    )
    .expect("render report");
    assert_eq!(report.schema_version, "bijux.bench.readiness.vcf_imputation_family_adapter.v1");
    assert_eq!(report.row_count, 8);
    assert_eq!(report.tool_count, 4);
    assert_eq!(report.benchmark_ready_row_count, 8);
    assert_eq!(report.parser_output_row_count, 8);
    assert_eq!(report.missing_input_test_passed_row_count, 8);
    assert!(report.rows.iter().any(|row| {
        row.tool_id == "minimac4"
            && row.stage_id == "vcf.impute"
            && row.panel_msav_path.as_deref().is_some_and(|path| path.ends_with("panel.msav"))
    }));
}
