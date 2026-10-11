//! Persisted BAM identity reports and schema identities.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const BAM_ARTIFACT_INVENTORY_SCHEMA_VERSION: &str = "bijux.bam.artifact_inventory.v1";

pub const BAM_SAMPLE_IDENTITY_SCHEMA_VERSION: &str = "bijux.bam.sample_identity.v1";

pub const BAM_REFERENCE_PREFLIGHT_SCHEMA_VERSION: &str = "bijux.bam.reference_preflight.v1";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BamArtifactEntryV1 {
    pub name: String,
    pub role: String,
    pub path: PathBuf,
    pub optional: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BamArtifactInventoryV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub stage_family: String,
    pub output_root: PathBuf,
    pub outputs: Vec<BamArtifactEntryV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BamSampleIdentityV1 {
    pub schema_version: String,
    pub sample_id: String,
    #[serde(default)]
    pub lane_id: Option<String>,
    #[serde(default)]
    pub library_id: Option<String>,
    #[serde(default)]
    pub platform: Option<String>,
    #[serde(default)]
    pub platform_unit: Option<String>,
    #[serde(default)]
    pub run_id: Option<String>,
    #[serde(default)]
    pub subject_id: Option<String>,
    #[serde(default)]
    pub cohort_id: Option<String>,
    #[serde(default)]
    pub read_group_policy: Option<String>,
    #[serde(default)]
    pub read_group_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BamReferenceAssetIdentityV1 {
    pub asset_kind: String,
    pub path: PathBuf,
    #[serde(default)]
    pub sha256: Option<String>,
    pub exists: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BamReferencePreflightV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub reference_fasta: PathBuf,
    #[serde(default)]
    pub reference_digest: Option<String>,
    pub contig_alias_policy: String,
    pub required_assets: Vec<BamReferenceAssetIdentityV1>,
    pub passes: bool,
    #[serde(default)]
    pub notes: Vec<String>,
}
