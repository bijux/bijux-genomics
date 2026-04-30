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

#[derive(Debug, Deserialize)]
pub(crate) struct AssetHydrationConfig {
    #[serde(default)]
    pub(crate) asset_bundle: Vec<AssetBundleEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct AssetBundleEntry {
    pub(crate) id: String,
    pub(crate) lock_family: String,
    #[serde(default)]
    pub(crate) stage_ids: Vec<String>,
    pub(crate) materialization_root: String,
    #[serde(default)]
    pub(crate) offline_replay_source: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct AssetLocksConfig {
    #[serde(default)]
    pub(crate) lock_family: Vec<AssetLockFamilyEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct AssetLockFamilyEntry {
    pub(crate) id: String,
    #[serde(default)]
    pub(crate) required_fields: Vec<String>,
}
