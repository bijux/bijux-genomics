#![cfg(test)]
//! Original scientific and binding behavior at its owning capability.
use crate::reference_panel::admission::bindings::resolve_governed_vcf_prepare_reference_panel_smoke_contract;
#[test]
fn governed_prepare_reference_panel_contract_uses_matrix_row() {
    let contract = resolve_governed_vcf_prepare_reference_panel_smoke_contract(
        &crate::test_support::inputs(),
        "bcftools",
    )
    .expect("resolve contract");
    assert_eq!(contract.stage, "vcf.prepare_reference_panel");
    assert_eq!(contract.tool, "bcftools");
    assert_eq!(contract.corpus, "vcf_production_regression");
    assert_eq!(contract.input_fixture, "reference_panel_sort_and_deduplicate");
    assert_eq!(contract.panel, "hsapiens_grch38_1kg_3202_chr22");
    assert_eq!(contract.map, "hsapiens_grch38_glimpse_chr22");
}
