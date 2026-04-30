use anyhow::{bail, Result};
use bijux_dna_db_ref::{
    ref_service, resolve_contig_map, resolve_reference_bank, resolve_species_context, ContigMap,
    MapCatalogEntry, PanelCatalogEntry, ReferenceBankEntry, ReferenceBundle,
    ResolvedSpeciesContext,
};
use bijux_dna_domain_vcf::contracts::{
    DefaultPanelSelectionPolicy, PanelSelectionPolicy, ReferencePanelGovernance,
};
use bijux_dna_domain_vcf::taxonomy::CoverageRegime;
use serde::Serialize;

use crate::api::{VcfPanelLock, VcfPipelineInputs};
use crate::coverage::classify_coverage_regime;

#[derive(Debug, Clone)]
pub struct ResolvedPlanningContext {
    pub species: ResolvedSpeciesContext,
    pub bundle: ReferenceBundle,
    pub reference_bank: ReferenceBankEntry,
    pub contig_map: ContigMap,
    pub panel_catalog: PanelCatalogEntry,
    pub map_catalog: MapCatalogEntry,
    pub resolved_coverage: CoverageRegime,
    pub selected_panel: Option<VcfPanelLock>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ReferenceContextReport {
    pub schema_version: String,
    pub species_id: String,
    pub build_id: String,
    pub bundle_id: String,
    pub bundle_lock_sha256: String,
    pub fasta_sha256: String,
    pub contig_naming_scheme: String,
    pub alias_count: usize,
    pub normalization_policy: String,
    pub panel_id: String,
    pub map_id: String,
    pub vcf_index_required: bool,
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
    let reference_bank = resolve_reference_bank(
        &inputs.species_context.species_id,
        &inputs.species_context.build_id,
    )?;
    let bundle = refs.resolve_reference_bundle(
        &inputs.species_context.species_id,
        &inputs.species_context.build_id,
    )?;
    let contig_map = resolve_contig_map(
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
    if reference_bank.build_id != inputs.species_context.build_id
        || reference_bank.species_id != inputs.species_context.species_id
    {
        bail!("reference bank drift detected: species/build does not match VCF planner inputs");
    }
    if contig_map.build_id != inputs.species_context.build_id
        || contig_map.species_id != inputs.species_context.species_id
    {
        bail!("contig map drift detected: species/build does not match VCF planner inputs");
    }
    if panel_catalog.build_id != inputs.species_context.build_id {
        bail!("panel catalog drift detected: build does not match VCF planner inputs");
    }
    if map_catalog.build_id != inputs.species_context.build_id {
        bail!("map catalog drift detected: build does not match VCF planner inputs");
    }
    let selected_panel = resolve_panel_lock(inputs)?;
    if let Some(panel) = &selected_panel {
        if panel.reference_build != inputs.species_context.build_id {
            bail!("panel lock drift detected: selected panel build does not match planner build");
        }
    }
    Ok(ResolvedPlanningContext {
        species,
        bundle,
        reference_bank,
        contig_map,
        panel_catalog,
        map_catalog,
        resolved_coverage,
        selected_panel,
    })
}

#[must_use]
pub fn reference_context_report(context: &ResolvedPlanningContext) -> ReferenceContextReport {
    ReferenceContextReport {
        schema_version: "bijux.vcf.reference_context_report.v1".to_string(),
        species_id: context.species.context.species_id.clone(),
        build_id: context.species.context.build_id.clone(),
        bundle_id: context.bundle.bundle_id.clone(),
        bundle_lock_sha256: context.bundle.bundle_lock_sha256.clone(),
        fasta_sha256: context.reference_bank.fasta_sha256.clone(),
        contig_naming_scheme: context.contig_map.naming_convention.clone(),
        alias_count: context.contig_map.aliases.len(),
        normalization_policy: format!("{:?}", context.bundle.normalization_policy),
        panel_id: context.panel_catalog.id.clone(),
        map_id: context.map_catalog.id.clone(),
        vcf_index_required: true,
    }
}

/// # Errors
/// Returns an error if the reference context cannot be resolved for the planner inputs.
pub fn resolve_reference_context_report(inputs: &VcfPipelineInputs) -> Result<ReferenceContextReport> {
    resolve(inputs).map(|context| reference_context_report(&context))
}
