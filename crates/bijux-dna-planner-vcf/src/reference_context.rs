use anyhow::{bail, Result};
use bijux_dna_db_ref::{
    ref_service, resolve_species_context, MapCatalogEntry, PanelCatalogEntry, ReferenceBundle,
    ResolvedSpeciesContext,
};
use bijux_dna_domain_vcf::contracts::{
    DefaultPanelSelectionPolicy, PanelSelectionPolicy, ReferencePanelGovernance,
};
use bijux_dna_domain_vcf::taxonomy::CoverageRegime;

use crate::api::{VcfPanelLock, VcfPipelineInputs};
use crate::coverage::classify_coverage_regime;

#[derive(Debug, Clone)]
pub struct ResolvedPlanningContext {
    pub species: ResolvedSpeciesContext,
    pub bundle: ReferenceBundle,
    pub panel_catalog: PanelCatalogEntry,
    pub map_catalog: MapCatalogEntry,
    pub resolved_coverage: CoverageRegime,
    pub selected_panel: Option<VcfPanelLock>,
}

/// # Errors
/// Returns an error if panel locks cannot be resolved against planner policy.
pub fn resolve_panel_lock(inputs: &VcfPipelineInputs) -> Result<Option<VcfPanelLock>> {
    if inputs
        .requested_stages
        .as_ref()
        .is_some_and(|stages| !stages.iter().any(|stage| stage == "vcf.prepare_reference_panel"))
    {
        return Ok(None);
    }
    let policy = DefaultPanelSelectionPolicy;
    let governance = inputs
        .panel_locks
        .iter()
        .map(|lock| ReferencePanelGovernance {
            panel_id: lock.panel_id.clone(),
            reference_build: lock.reference_build.clone(),
            panel_checksum_sha256: lock.panel_checksum_sha256.clone(),
            index_checksum_sha256: lock.index_checksum_sha256.clone(),
            license_id: lock.license_id.clone(),
            license_constraints: vec![],
            ancestry_tags: vec![],
            target_tags: vec![],
        })
        .collect::<Vec<_>>();

    let selected = policy.select_panel(&governance, &inputs.panel_selection);
    Ok(selected.map(|entry| VcfPanelLock {
        panel_id: entry.panel_id.clone(),
        reference_build: entry.reference_build.clone(),
        panel_checksum_sha256: entry.panel_checksum_sha256.clone(),
        index_checksum_sha256: entry.index_checksum_sha256.clone(),
        license_id: entry.license_id.clone(),
    }))
}

/// # Errors
/// Returns an error if species, bundle, panel, map, or coverage context cannot be resolved.
pub fn resolve(inputs: &VcfPipelineInputs) -> Result<ResolvedPlanningContext> {
    let refs = ref_service();
    let species = resolve_species_context(
        &inputs.species_context.species_id,
        &inputs.species_context.build_id,
    )?;
    let bundle = refs.resolve_reference_bundle(
        &inputs.species_context.species_id,
        &inputs.species_context.build_id,
    )?;
    let panel_catalog = refs.resolve_panel(
        &inputs.species_context.species_id,
        &inputs.species_context.build_id,
        inputs.panel_id.as_deref(),
    )?;
    let map_catalog = refs.resolve_map(
        &inputs.species_context.species_id,
        &inputs.species_context.build_id,
        inputs.map_id.as_deref(),
    )?;
    let resolved_coverage_profile = refs.resolve_coverage_profile(
        &inputs.species_context.species_id,
        &inputs.species_context.build_id,
    )?;
    let (resolved_coverage, _coverage_reason, _thresholds) = classify_coverage_regime(
        inputs.coverage_regime,
        inputs.mean_depth_x,
        resolved_coverage_profile.as_deref(),
    )?;
    if species.context.contig_set_digest != bundle.contig_set_digest {
        bail!(
            "reference bundle drift detected: species context digest does not match bundle digest"
        );
    }
    let selected_panel = resolve_panel_lock(inputs)?;
    Ok(ResolvedPlanningContext {
        species,
        bundle,
        panel_catalog,
        map_catalog,
        resolved_coverage,
        selected_panel,
    })
}
