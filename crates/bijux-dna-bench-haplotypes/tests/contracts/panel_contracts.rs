//! Original scientific and binding behavior at its owning capability.
use crate::panel_contracts::governed_vcf_panel_species_context;
use crate::panel_contracts::resolve_governed_vcf_panel_asset_contract;
use crate::panel_contracts::resolve_governed_vcf_panel_workflow_smoke_contract;
use bijux_dna_bench_corpus::panel_metadata::materialize_local_vcf_panel_fixture;
#[test]
fn governed_panel_workflow_contract_accepts_canonical_tool_substitution() {
    let contract = resolve_governed_vcf_panel_workflow_smoke_contract(
        &crate::test_support::inputs(),
        "vcf.phasing",
        "shapeit5",
        "phased_vcf",
    )
    .expect("resolve contract");
    assert_eq!(contract.stage, "vcf.phasing");
    assert_eq!(contract.tool, "shapeit5");
    assert_eq!(contract.corpus, "vcf_production_regression");
    assert_eq!(contract.panel, "hsapiens_grch38_1kg_3202_chr22");
    assert_eq!(contract.map, "hsapiens_grch38_glimpse_chr22");

    let shapeit = resolve_governed_vcf_panel_workflow_smoke_contract(
        &crate::test_support::inputs(),
        "vcf.phasing",
        "shapeit",
        "phased_vcf",
    )
    .expect("resolve Shapeit4 contract");
    assert_eq!(shapeit.stage, "vcf.phasing");
    assert_eq!(shapeit.tool, "shapeit");
}

#[test]
fn governed_panel_workflow_contract_refuses_unknown_tool_binding() {
    let error = resolve_governed_vcf_panel_workflow_smoke_contract(
        &crate::test_support::inputs(),
        "vcf.phasing",
        "not-a-phasing-tool",
        "phased_vcf",
    )
    .expect_err("unknown tool must be refused");
    assert!(error.to_string().contains("has no canonical binding"));
}

#[test]
fn governed_imputation_contract_accepts_derived_minimac_panel() {
    let contract = resolve_governed_vcf_panel_workflow_smoke_contract(
        &crate::test_support::inputs(),
        "vcf.impute",
        "minimac4",
        "imputed_vcf",
    )
    .expect("Minimac4 must accept the governed derived MSAV panel");
    assert_eq!(contract.tool, "minimac4");
    assert_eq!(contract.panel, "hsapiens_grch38_1kg_3202_chr22");
    assert_eq!(contract.map, "hsapiens_grch38_glimpse_chr22");
}

#[test]
fn governed_panel_workflow_asset_contract_resolves_without_materialization() {
    let report = resolve_governed_vcf_panel_asset_contract().expect("resolve asset contract");
    assert_eq!(report.panel_id, "hsapiens_grch38_1kg_3202_chr22");
    assert_eq!(report.map_id, "hsapiens_grch38_glimpse_chr22");
    assert_eq!(report.fixture_class, "local_smoke_metadata_only");
    assert!(!report.panel_files.is_empty());
    assert!(!report.map_files.is_empty());
}

#[test]
fn local_panel_fixture_is_explicitly_classified() {
    let dir = tempfile::tempdir().expect("tempdir");
    let report = materialize_local_vcf_panel_fixture(dir.path()).expect("materialize fixture");
    assert_eq!(report.fixture_class, "local_smoke");
    assert!(report.materialized_files.iter().all(|path| std::path::Path::new(path).is_file()));
    let derived_msav = report
        .materialized_files
        .iter()
        .find(|path| path.ends_with("/panel.msav"))
        .expect("derived MSAV fixture");
    let marker = std::fs::read_to_string(derived_msav).expect("read derived MSAV marker");
    assert!(marker.contains("fixture_class=local_smoke"));
    assert!(marker.contains("derived_from=panel_vcf"));
    assert!(marker.contains("derivation_tool=minimac4"));
    assert!(marker.contains("--compress-reference panel.vcf.gz --output panel.msav"));
}

#[test]
fn governed_panel_species_context_keeps_chr_and_numeric_aliases() {
    let context = governed_vcf_panel_species_context();
    assert_eq!(context.species_id, "Homo sapiens");
    assert_eq!(context.build_id, "GRCh38");
    assert!(context.contigs.iter().any(|contig| contig.name == "1"));
    assert!(context.contigs.iter().any(|contig| contig.name == "chr1"));
}
