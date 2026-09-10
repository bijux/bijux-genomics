//! Materialize only admitted metadata markers; never replace reference asset bytes.
use super::admission::resolve_governed_vcf_panel_asset_contract;
use super::markers::{admit_markers, plan_markers};
use super::model::LocalVcfPanelFixtureReport;
use anyhow::Result;
use std::path::Path;

/// Materialize metadata-only panel/map fixtures, reusing byte-identical markers.
///
/// # Errors
/// Rejects unsafe paths and existing non-marker assets before writing any marker.
/// Publication does not replace files and is not a multi-file transaction.
pub fn materialize_local_vcf_panel_fixture(
    materialization_root: &Path,
) -> Result<LocalVcfPanelFixtureReport> {
    let contract = resolve_governed_vcf_panel_asset_contract()?;
    let markers = plan_markers(materialization_root, &contract)?;
    let absent = admit_markers(&markers)?;
    for marker in absent {
        // Create without replacement: a concurrent writer cannot lose its asset bytes.
        bijux_dna_infra::atomic_create_bytes(&marker.path, marker.payload.as_bytes())?;
    }
    Ok(LocalVcfPanelFixtureReport {
        schema_version: "bijux.local_vcf_panel_fixture.v1",
        fixture_class: "local_smoke",
        panel_id: contract.panel_id,
        map_id: contract.map_id,
        materialized_files: markers
            .iter()
            .map(|marker| marker.path.display().to_string())
            .collect(),
    })
}
