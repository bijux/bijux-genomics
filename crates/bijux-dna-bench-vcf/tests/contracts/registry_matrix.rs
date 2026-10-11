use std::path::PathBuf;

use bijux_dna_bench_vcf::evidence::registry::matrix::{
    render_vcf_matrix_registry_consistency, DEFAULT_VCF_MATRIX_REGISTRY_CONSISTENCY_PATH,
    VCF_MATRIX_REGISTRY_CONSISTENCY_SCHEMA_VERSION,
};

fn repo_root() -> PathBuf {
    crate::support::repo_root()
}

#[test]
fn vcf_matrix_registry_consistency_gate_reports_governed_pass_state() {
    let root = repo_root();
    let report = render_vcf_matrix_registry_consistency(
        &crate::support::inputs(&root),
        &root,
        &PathBuf::from(DEFAULT_VCF_MATRIX_REGISTRY_CONSISTENCY_PATH),
    )
    .expect("render VCF matrix/registry consistency");

    assert_eq!(report.schema_version, VCF_MATRIX_REGISTRY_CONSISTENCY_SCHEMA_VERSION);
    assert_eq!(report.domain, "vcf");
    assert!(report.passes_gate);
    assert_eq!(report.stage_count, 20);
    assert_eq!(report.matrix_row_count, crate::support::inputs(&root).bindings().len());
    assert_eq!(report.registry_pair_count, 39);
    assert_eq!(report.benchmark_ready_registry_pair_count, 39);
    assert_eq!(report.unregistered_matrix_pair_count, 0);
    assert_eq!(report.missing_benchmark_ready_registry_pair_count, 0);
    assert!(report.rows.is_empty());
}
