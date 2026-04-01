use std::collections::BTreeMap;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct BundlesConfig {
    #[serde(default)]
    pub(crate) bundle: Vec<BundleEntry>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct BundleEntry {
    pub(crate) bundle_id: String,
    pub(crate) species_id: String,
    pub(crate) build_id: String,
    pub(crate) fasta: String,
    pub(crate) fai: String,
    pub(crate) dict: String,
    pub(crate) contig_set_digest: String,
    #[serde(default)]
    pub(crate) mask_bed: Option<String>,
    #[serde(default)]
    pub(crate) regions_bed: Option<String>,
    pub(crate) source_lock_sha256: String,
    pub(crate) bundle_lock_sha256: String,
    pub(crate) normalization_policy: String,
    #[serde(default)]
    pub(crate) remap: BTreeMap<String, String>,
    pub(crate) sex_system: String,
    pub(crate) par_policy: String,
    #[serde(default)]
    pub(crate) default_coverage_regime: Option<String>,
    #[serde(default)]
    pub(crate) supported_features: SupportedFeatureEntry,
    pub(crate) contigs: Vec<ContigEntry>,
}

#[derive(Debug, Default, Deserialize)]
pub(crate) struct SupportedFeatureEntry {
    #[serde(default)]
    pub(crate) sex_chr: bool,
    #[serde(default)]
    pub(crate) imputation: bool,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ContigEntry {
    pub(crate) name: String,
    pub(crate) length_bp: u64,
}
