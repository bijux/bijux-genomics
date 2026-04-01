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
