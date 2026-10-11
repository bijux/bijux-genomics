#![cfg(test)]
use crate::filter::admission::bindings::resolve_governed_vcf_filter_smoke_contract;
use crate::filter::fixtures::inputs::write_governed_filter_input_vcf;
use crate::filter::observations::variants::parse_vcf_record_count;

#[test]
fn governed_filter_contract_uses_single_sample_matrix_row() {
    let contract =
        resolve_governed_vcf_filter_smoke_contract(&crate::test_support::inputs(), "bcftools")
            .expect("resolve contract");
    assert_eq!(contract.stage_id, "vcf.filter");
    assert_eq!(contract.tool_id, "bcftools");
    assert_eq!(contract.corpus_id, "vcf_production_regression");
    assert_eq!(contract.input_fixture_id, "site_filter_single_sample");
    assert_eq!(contract.sample_name, "sample_a");
}

#[test]
fn governed_filter_fixture_contains_five_records() {
    let dir = tempfile::tempdir().expect("tempdir");
    let input_vcf = dir.path().join("filter_input.vcf");
    write_governed_filter_input_vcf(&input_vcf).expect("write governed input");
    let record_count = parse_vcf_record_count(&input_vcf).expect("count records");
    assert_eq!(record_count, 5);
}
