use std::collections::BTreeMap;

use bijux_dna_domain_vcf::contracts::SpeciesContext;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct BuildId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContigMap {
    pub species_id: String,
    pub build_id: String,
    pub naming_convention: String,
    pub mitochondrion_id: String,
    #[serde(default)]
    pub aliases: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpeciesAuthorityEntry {
    pub species_id: String,
    pub default_build_id: String,
    pub contig_naming: String,
    pub sex_chromosomes: String,
    pub mitochondrion_id: String,
    pub ploidy_model: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReferenceBankEntry {
    pub species_id: String,
    pub build_id: String,
    pub fasta_url: String,
    pub fasta_sha256: String,
    pub license_id: String,
    pub license_url: String,
    #[serde(default)]
    pub required_indexes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeneticMapBankEntry {
    pub id: String,
    pub species_id: String,
    pub build_id: String,
    #[serde(default)]
    pub panel_id: Option<String>,
    pub map_id: String,
    pub map_url: String,
    pub map_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParRegion {
    pub contig: String,
    pub start_bp: u64,
    pub end_bp: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SexChromosomeRule {
    pub species_id: String,
    pub build_id: String,
    pub male_x_ploidy: u8,
    pub male_y_ploidy: u8,
    #[serde(default)]
    pub par_regions: Vec<ParRegion>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrganellarPolicy {
    pub species_id: String,
    pub build_id: String,
    pub mitochondrion_id: String,
    #[serde(default)]
    pub chloroplast_id: Option<String>,
    #[serde(default)]
    pub chloroplast_required_for_profiles: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReferenceSet {
    pub id: String,
    pub species_id: String,
    pub usecase: String,
    pub primary_reference: String,
    #[serde(default)]
    pub contaminants: Vec<String>,
    #[serde(default)]
    pub spike_in: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ContigNormalizationPolicy {
    StrictOnly,
    DeterministicRemap,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ResolvedSpeciesContext {
    pub context: SpeciesContext,
    pub supported_features: SupportedFeatures,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SupportedFeatures {
    pub sex_chr: bool,
    pub imputation: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReferenceBundle {
    pub bundle_id: String,
    pub species_id: String,
    pub build_id: String,
    pub fasta: String,
    pub fai: String,
    pub dict: String,
    pub contig_set_digest: String,
    pub mask_bed: Option<String>,
    pub regions_bed: Option<String>,
    pub source_lock_sha256: String,
    pub bundle_lock_sha256: String,
    pub normalization_policy: ContigNormalizationPolicy,
    pub remap_table: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReferenceProvenance {
    pub species_id: String,
    pub build_id: String,
    pub bundle_id: String,
    pub contig_set_digest: String,
    pub source_lock_sha256: String,
    pub bundle_lock_sha256: String,
}
