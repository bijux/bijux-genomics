//! Corpus panel metadata admission.
use super::model::ResolvedVcfPanelAssetContract;
use super::model::ResolvedVcfPanelDerivationContract;
use super::model::GOVERNED_VCF_MAP_ID;
use super::model::GOVERNED_VCF_PANEL_BUILD_ID;
use super::model::GOVERNED_VCF_PANEL_ID;
use super::model::GOVERNED_VCF_PANEL_SPECIES_ID;
use anyhow::Result;
use bijux_dna_db_ref::public_api::resolve_map;
use bijux_dna_db_ref::public_api::resolve_map_lock;
use bijux_dna_db_ref::public_api::resolve_panel;
use bijux_dna_db_ref::public_api::resolve_panel_lock;
use bijux_dna_db_ref::public_api::validate_imputation_tool_compatibility;

/// Resolve or materialize the governed local panel metadata fixture.
///
/// # Errors
/// Returns an error for invalid reference authority, incompatible assets, poisoned locks or failed IO.
pub fn resolve_governed_vcf_panel_asset_contract() -> Result<ResolvedVcfPanelAssetContract> {
    let panel = resolve_panel(
        GOVERNED_VCF_PANEL_SPECIES_ID,
        GOVERNED_VCF_PANEL_BUILD_ID,
        Some(GOVERNED_VCF_PANEL_ID),
    )?;
    let map = resolve_map(
        GOVERNED_VCF_PANEL_SPECIES_ID,
        GOVERNED_VCF_PANEL_BUILD_ID,
        Some(GOVERNED_VCF_MAP_ID),
    )?;
    let panel_lock = resolve_panel_lock(&panel)?;
    let map_lock = resolve_map_lock(&map)?;
    validate_imputation_tool_compatibility("glimpse", &panel, &map)?;
    if panel.compatibility.minimac_reference_format == "msav" {
        validate_imputation_tool_compatibility("minimac4", &panel, &map)?;
    }

    let panel_derivations = panel_lock
        .files
        .iter()
        .filter_map(|file| {
            file.derivation.as_ref().map(|derivation| ResolvedVcfPanelDerivationContract {
                path: file.path.clone(),
                source_file_name: derivation.source_file_name.clone(),
                tool_id: derivation.tool_id.clone(),
                tool_version: derivation.tool_version.clone(),
                tool_artifact_sha256: derivation.tool_artifact_sha256.clone(),
                argv: derivation.argv.clone(),
            })
        })
        .collect::<Vec<_>>();

    Ok(ResolvedVcfPanelAssetContract {
        schema_version: "bijux.local_vcf_panel_asset_contract.v1",
        fixture_class: "local_smoke_metadata_only",
        species_id: panel.species_id,
        build_id: panel.build_id,
        panel_id: panel.id,
        map_id: map.id,
        panel_files: panel_lock.files.into_iter().map(|file| file.path).collect(),
        panel_derivations,
        map_files: map_lock.files.into_iter().map(|file| file.path).collect(),
    })
}
