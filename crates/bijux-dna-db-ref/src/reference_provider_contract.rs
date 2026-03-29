use super::*;

#[test]
fn species_context_and_bundle_resolve() {
    let resolved = resolve_species_context("Homo sapiens", "GRCh38")
        .unwrap_or_else(|err| panic!("resolve species context: {err}"));
    assert_eq!(resolved.context.build_id, "GRCh38");
    assert!(resolved.supported_features.imputation);
    let bundle = resolve_reference_bundle("Homo sapiens", "GRCh38")
        .unwrap_or_else(|err| panic!("resolve reference bundle: {err}"));
    assert_eq!(bundle.bundle_id, "hsapiens_grch38_primary");
    let authority = resolve_species_authority("Homo sapiens")
        .unwrap_or_else(|err| panic!("resolve species authority: {err}"));
    assert_eq!(authority.default_build_id, "GRCh38");
    let bank = resolve_reference_bank("Homo sapiens", "GRCh38")
        .unwrap_or_else(|err| panic!("resolve reference bank: {err}"));
    assert!(!bank.required_indexes.is_empty());
}

#[test]
fn deterministic_remap_table_is_enforced() {
    let bundle = resolve_reference_bundle("Canis lupus", "CanFam4")
        .unwrap_or_else(|err| panic!("resolve reference bundle: {err}"));
    let mapped = normalize_contig_name(&bundle, "chr1")
        .unwrap_or_else(|err| panic!("normalize contig: {err}"));
    assert_eq!(mapped, "1");
}

#[test]
fn panel_and_map_resolution_work() {
    let panel = resolve_panel("Homo sapiens", "GRCh38", None)
        .unwrap_or_else(|err| panic!("resolve panel: {err}"));
    let map = resolve_map("Homo sapiens", "GRCh38", None)
        .unwrap_or_else(|err| panic!("resolve map: {err}"));
    let panel_lock =
        resolve_panel_lock(&panel).unwrap_or_else(|err| panic!("resolve panel lock: {err}"));
    let map_lock = resolve_map_lock(&map).unwrap_or_else(|err| panic!("resolve map lock: {err}"));
    assert!(!panel_lock.files.is_empty());
    assert!(!map_lock.files.is_empty());
    validate_imputation_tool_compatibility("glimpse", &panel, &map)
        .unwrap_or_else(|err| panic!("compatibility: {err}"));
}

#[test]
fn minimac_requires_m3vcf_support() {
    let panel = resolve_panel("Homo sapiens", "GRCh38", Some("hsapiens_grch38_full"))
        .unwrap_or_else(|err| panic!("resolve panel: {err}"));
    let map = resolve_map("Homo sapiens", "GRCh38", Some("hsapiens_grch38_chr_map"))
        .unwrap_or_else(|err| panic!("resolve map: {err}"));
    let err = match validate_imputation_tool_compatibility("minimac4", &panel, &map) {
        Ok(()) => panic!("full panel must refuse minimac4"),
        Err(err) => err,
    };
    assert!(err.to_string().contains("minimac4"));
}

#[test]
fn invalid_lock_ref_is_rejected() {
    let panel = PanelCatalogEntry {
        id: "panel".to_string(),
        species_id: "Homo sapiens".to_string(),
        build_id: "GRCh38".to_string(),
        status: "production".to_string(),
        version: "1.0.0".to_string(),
        license: "CC-BY-4.0".to_string(),
        lock_ref: "not_a_lock_ref".to_string(),
        citation: None,
        files: vec![],
        compatibility: CatalogCompatibility {
            tool_tags: vec!["glimpse".to_string()],
            requires_phased: true,
            supports_gl_input: true,
            supports_minimac_m3vcf: false,
            glimpse_reference_format: "bcf+sites".to_string(),
        },
    };
    let Err(err) = resolve_panel_lock(&panel) else {
        panic!("invalid lock_ref must fail");
    };
    assert!(err.to_string().contains("invalid lock_ref"));
}

#[test]
fn declared_build_guard_rejects_mismatch() {
    let Err(err) = enforce_declared_build_and_contigs(
        "Homo sapiens",
        "GRCh37",
        &["chr1".to_string(), "chr2".to_string()],
    ) else {
        panic!("mismatch must fail");
    };
    assert!(err.to_string().contains("declared build mismatch"));
}

#[test]
fn map_sex_organellar_and_reference_set_resolve() {
    let map = resolve_genetic_map_bank("Homo sapiens", "GRCh38", None)
        .unwrap_or_else(|err| panic!("resolve genetic map bank: {err}"));
    assert_eq!(map.map_id, "hsapiens_grch38_chr_map");
    let sex = resolve_sex_chromosome_rule("Homo sapiens", "GRCh38")
        .unwrap_or_else(|err| panic!("resolve sex chromosome rule: {err}"));
    assert!(!sex.par_regions.is_empty());
    let organellar = resolve_organellar_policy("Homo sapiens", "GRCh38")
        .unwrap_or_else(|err| panic!("resolve organellar policy: {err}"));
    assert_eq!(organellar.mitochondrion_id, "MT");
    let refs = resolve_default_reference_set("Homo sapiens", "adna")
        .unwrap_or_else(|err| panic!("resolve default reference set: {err}"));
    assert_eq!(refs.primary_reference, "hsapiens_grch38_primary");
}
