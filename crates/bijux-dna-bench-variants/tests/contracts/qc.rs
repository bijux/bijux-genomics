use crate::qc::admission::bindings::resolve_governed_vcf_qc_smoke_contract;
use crate::qc::fixtures::inputs::write_governed_qc_input_vcf;
use crate::qc::workflow::run_local_vcf_qc_smoke;

#[test]
fn governed_qc_contract_uses_cohort_matrix_row() {
    let contract = resolve_governed_vcf_qc_smoke_contract(&crate::test_support::inputs(), "plink2")
        .expect("resolve contract");
    assert_eq!(contract.stage_id, "vcf.qc");
    assert_eq!(contract.tool_id, "plink2");
    assert_eq!(contract.corpus_id, "vcf_production_regression");
    assert_eq!(contract.input_fixture_id, "qc_cohort_missingness");
    assert_eq!(contract.sample_name, "qc_cohort");
}

#[test]
#[allow(clippy::float_cmp)] // Exact governed exclusion thresholds are part of this contract.
fn governed_qc_fixture_surfaces_named_exclusions() {
    let repo_root = tempfile::tempdir().expect("tempdir");
    let report = run_local_vcf_qc_smoke(&crate::test_support::inputs(), repo_root.path(), "plink2")
        .expect("run local smoke");
    assert_eq!(report.stage_id, "vcf.qc");
    assert_eq!(report.tool_id, "plink2");
    assert_eq!(report.sample_missingness_exclusion_threshold, 0.5);
    assert_eq!(report.variant_missingness_exclusion_threshold, 0.5);
    assert_eq!(report.excluded_samples.len(), 1);
    assert_eq!(report.excluded_samples[0].sample_id, "qc_sparse");
    assert_eq!(report.excluded_variants.len(), 1);
    assert_eq!(report.excluded_variants[0].variant_id, "chr1:30:G:A");
    assert_eq!(report.maf_summary.observed_variant_count, 4);
    assert_eq!(report.heterozygosity.het_hom_ratio, Some(2.0));
    assert_eq!(report.hwe_summary.tested_variant_count, 3);
    assert_eq!(report.hwe_summary.pvalue_mean, Some(0.825_656));
    assert_eq!(report.hwe_summary.status, "computed_modern");
}

#[test]
fn governed_qc_fixture_writer_creates_named_samples() {
    let dir = tempfile::tempdir().expect("tempdir");
    let input_vcf = dir.path().join("qc_input.vcf");
    write_governed_qc_input_vcf(&input_vcf).expect("write governed input");
    let raw = std::fs::read_to_string(&input_vcf).expect("read input");
    assert!(raw.contains("\tqc_ref\tqc_sparse\tqc_balanced\n"));
    assert!(raw.contains("chr1\t30\t.\tG\tA\t59\tLOWQUAL"));
}
