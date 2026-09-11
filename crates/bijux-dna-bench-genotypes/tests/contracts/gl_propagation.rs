use crate::gl_propagation::admission::bindings::resolve_governed_vcf_gl_propagation_smoke_contract;
use crate::gl_propagation::fixtures::inputs::write_governed_gl_propagation_input_vcf;
use crate::gl_propagation::observations::variants::summarize_likelihood_fields;
use std::path::PathBuf;

#[test]
fn governed_gl_propagation_contract_uses_single_sample_matrix_row() {
    let contract = resolve_governed_vcf_gl_propagation_smoke_contract(
        &crate::test_support::inputs(),
        "bcftools",
    )
    .expect("resolve governed gl propagation contract");
    assert_eq!(contract.stage_id, "vcf.gl_propagation");
    assert_eq!(contract.tool_id, "bcftools");
    assert_eq!(contract.corpus_id, "vcf_production_regression");
    assert_eq!(contract.input_fixture_id, "likelihood_single_sample");
    assert_eq!(contract.sample_name, "sample_a");
}

#[test]
fn governed_gl_fixture_retains_all_likelihood_fields() {
    let dir = tempfile::tempdir().expect("tempdir");
    let input_vcf = dir.path().join("gl_input.vcf");
    write_governed_gl_propagation_input_vcf(&input_vcf).expect("write governed input");
    let summary = summarize_likelihood_fields(&input_vcf).expect("summarize fields");
    assert_eq!(summary.site_count, 3);
    assert_eq!(
        summary.fields.into_iter().collect::<Vec<_>>(),
        vec!["GL".to_string(), "GP".to_string(), "PL".to_string()]
    );
}

fn repo_root() -> PathBuf {
    crate::test_support::repo_root()
}

#[test]
fn summarize_likelihood_fields_reads_governed_single_sample_fixture() {
    let repo_root = repo_root();
    let fixture_vcf = repo_root
        .join("benchmarks/tests/fixtures/corpora/vcf-mini/variants/vcf_mini_raw_single_sample.vcf");
    let summary = summarize_likelihood_fields(&fixture_vcf).expect("summarize fixture fields");
    assert_eq!(summary.site_count, 2);
    assert!(summary.fields.is_empty());
}
