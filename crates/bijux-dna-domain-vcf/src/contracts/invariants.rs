use std::collections::BTreeSet;

use anyhow::{bail, Result};
use serde::Serialize;

use crate::taxonomy::{CoverageRegime, VcfDomainStage};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SpeciesContext {
    pub species_id: String,
    pub build_id: String,
    pub contig_set_digest: String,
    pub contigs: Vec<ContigSpec>,
    pub sex_system: String,
    pub par_policy: String,
    pub default_coverage_regime: Option<CoverageRegime>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ContigSpec {
    pub name: String,
    pub length_bp: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[allow(clippy::struct_excessive_bools)]
pub struct EntryVcfInvariantState {
    pub build_id: String,
    pub contig_set_digest: String,
    pub sorted_by_contig_and_pos: bool,
    pub bgzip_compressed: bool,
    pub tabix_index_present: bool,
    pub sample_ids_non_empty_unique: bool,
    pub ploidy_constraints_ok: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[allow(clippy::struct_excessive_bools)]
pub struct PanelMapInvariantState {
    pub species_id: String,
    pub build_id: String,
    pub contig_set_digest: String,
    pub phased_or_gl_compatible: bool,
    pub format_requirements_ok: bool,
    pub sample_count_ok: bool,
    pub license_allowed: bool,
    pub checksums_match: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum RefusalReason {
    BuildMismatch,
    ContigMismatch,
    LowOverlap,
    UnsupportedSexParPolicy,
    MissingBackendRequiredFields,
    UnsupportedPseudohaploidToDiploid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[allow(clippy::struct_excessive_bools)]
pub struct VcfInvariantState {
    pub sorted_by_contig_and_pos: bool,
    pub bgzip_compressed: bool,
    pub tabix_index_present: bool,
    pub sample_set_consistent: bool,
    pub contig_set_consistent: bool,
}

/// # Errors
/// Returns an error when required VCF invariants are violated for the stage.
pub fn validate_vcf_invariants(
    stage: VcfDomainStage,
    invariants: &VcfInvariantState,
) -> Result<()> {
    if !invariants.sorted_by_contig_and_pos {
        bail!("{} requires sorted VCF records", stage.as_str());
    }
    if !invariants.sample_set_consistent {
        bail!("{} requires sample consistency across inputs", stage.as_str());
    }
    if !invariants.contig_set_consistent {
        bail!("{} requires contig consistency across inputs", stage.as_str());
    }

    let requires_bgzip = matches!(
        stage,
        VcfDomainStage::Filter
            | VcfDomainStage::GlPropagation
            | VcfDomainStage::Phasing
            | VcfDomainStage::ImputationMetrics
            | VcfDomainStage::Impute
            | VcfDomainStage::Postprocess
            | VcfDomainStage::Stats
    );
    if requires_bgzip && !invariants.bgzip_compressed {
        bail!("{} requires bgzip-compressed VCF", stage.as_str());
    }
    if requires_bgzip && !invariants.tabix_index_present {
        bail!("{} requires tabix index", stage.as_str());
    }
    Ok(())
}

/// # Errors
/// Returns an error when species context is incomplete.
pub fn validate_species_context(species: &SpeciesContext) -> Result<()> {
    if species.species_id.trim().is_empty()
        || species.build_id.trim().is_empty()
        || species.contig_set_digest.trim().is_empty()
    {
        bail!("species context requires species_id/build_id/contig_set_digest");
    }
    if species.contigs.is_empty() {
        bail!("species context requires non-empty contig list");
    }
    let mut seen_contigs = BTreeSet::new();
    for contig in &species.contigs {
        if contig.name.trim().is_empty() {
            bail!("species context contig names must be non-empty");
        }
        if contig.length_bp == 0 {
            bail!("species context contig lengths must be positive");
        }
        if !seen_contigs.insert(contig.name.as_str()) {
            bail!("species context contig names must be unique");
        }
    }
    if species.par_policy.eq_ignore_ascii_case("unsupported")
        && !species.sex_system.eq_ignore_ascii_case("unknown")
    {
        bail!("unsupported PAR policy must use sex_system=unknown");
    }
    Ok(())
}

/// # Errors
/// Returns an error if entry VCF invariants are violated for a species context.
pub fn validate_entry_vcf_invariants(
    species: &SpeciesContext,
    state: &EntryVcfInvariantState,
) -> Result<()> {
    validate_species_context(species)?;
    if state.build_id != species.build_id {
        bail!("{:?}", RefusalReason::BuildMismatch);
    }
    if state.contig_set_digest != species.contig_set_digest {
        bail!("{:?}", RefusalReason::ContigMismatch);
    }
    if !state.sorted_by_contig_and_pos {
        bail!("entry VCF must be sorted");
    }
    if !state.bgzip_compressed || !state.tabix_index_present {
        bail!("entry VCF must be bgzip + tabix indexed");
    }
    if !state.sample_ids_non_empty_unique {
        bail!("entry VCF sample IDs must be unique and non-empty");
    }
    if !state.ploidy_constraints_ok {
        bail!("entry VCF ploidy constraints failed");
    }
    Ok(())
}

/// # Errors
/// Returns an error if panel/map invariants violate species/build compatibility.
pub fn validate_panel_map_invariants(
    species: &SpeciesContext,
    state: &PanelMapInvariantState,
) -> Result<()> {
    validate_species_context(species)?;
    if state.species_id != species.species_id || state.build_id != species.build_id {
        bail!("{:?}", RefusalReason::BuildMismatch);
    }
    if state.contig_set_digest != species.contig_set_digest {
        bail!("{:?}", RefusalReason::ContigMismatch);
    }
    if !state.phased_or_gl_compatible
        || !state.format_requirements_ok
        || !state.sample_count_ok
        || !state.license_allowed
        || !state.checksums_match
    {
        bail!("panel/map invariants failed");
    }
    Ok(())
}

/// # Errors
/// Returns an error when pseudo-haploid to diploid conversion is requested.
pub fn refuse_unsupported_regime_transition(
    coverage: CoverageRegime,
    requires_diploid_imputation: bool,
) -> Result<()> {
    if coverage == CoverageRegime::Pseudohaploid && requires_diploid_imputation {
        bail!("{:?}", RefusalReason::UnsupportedPseudohaploidToDiploid);
    }
    Ok(())
}
