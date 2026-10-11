use serde::Deserialize;

use crate::{GeneticMapBankEntry, OrganellarPolicy, ReferenceBankEntry, ReferenceSet};

#[derive(Debug, Deserialize)]
pub(crate) struct ReferenceBankConfig {
    #[serde(default)]
    pub(crate) reference: Vec<ReferenceBankEntry>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct GeneticMapBankConfig {
    #[serde(default)]
    pub(crate) map: Vec<GeneticMapBankEntry>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct OrganellarPolicyConfig {
    #[serde(default)]
    pub(crate) policy: Vec<OrganellarPolicy>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ReferenceSetConfig {
    #[serde(default)]
    pub(crate) set: Vec<ReferenceSet>,
}
