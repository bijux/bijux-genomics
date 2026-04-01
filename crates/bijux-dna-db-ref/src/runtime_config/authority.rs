use std::collections::BTreeMap;

use serde::Deserialize;

use crate::{ContigMap, SexChromosomeRule, SpeciesAuthorityEntry};

#[derive(Debug, Deserialize)]
pub(crate) struct AliasesConfig {
    #[serde(default)]
    pub(crate) aliases: BTreeMap<String, String>,
    #[serde(default)]
    pub(crate) default_builds: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CoverageRegimesConfig {
    #[serde(default)]
    pub(crate) species_profile: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct SpeciesAuthorityConfig {
    #[serde(default)]
    pub(crate) species: Vec<SpeciesAuthorityEntry>,
    #[serde(default)]
    pub(crate) contig_map: Vec<ContigMap>,
    #[serde(default)]
    pub(crate) sex_rule: Vec<SexChromosomeRule>,
}
