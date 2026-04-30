use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

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
pub struct ReferenceBundle {
    pub bundle_id: String,
    pub species_id: String,
    pub build_id: String,
    pub fasta: String,
    pub fai: String,
    pub dict: String,
    pub contig_set_digest: String,
    pub contigs: Vec<String>,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MaterializedIndexArtifact {
    pub tool_id: String,
    pub path: PathBuf,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReferenceMaterializationReport {
    pub schema_version: String,
    pub species_id: String,
    pub build_id: String,
    pub source_url: String,
    pub declared_sha256: String,
    pub license_id: String,
    pub license_url: String,
    pub materialization_root: PathBuf,
    pub mode: String,
    pub bundle_id: String,
    pub index_artifacts: Vec<MaterializedIndexArtifact>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReferenceBundleResolverReport {
    pub schema_version: String,
    pub species_id: String,
    pub build_id: String,
    pub bundle_id: String,
    pub contig_aliases: BTreeMap<String, String>,
    pub panel_id: Option<String>,
    pub map_id: Option<String>,
    pub map_bank_id: Option<String>,
    pub compatibility_checked_tool: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReferenceIndexQaReport {
    pub schema_version: String,
    pub species_id: String,
    pub build_id: String,
    pub materialization_root: PathBuf,
    pub verified_artifacts: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VcfPanelMaterializationReport {
    pub schema_version: String,
    pub species_id: String,
    pub build_id: String,
    pub panel_id: String,
    pub map_id: String,
    pub materialization_root: PathBuf,
    pub compatible_tool_tags: Vec<String>,
    pub materialized_files: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MaterializedDbBundle {
    pub bundle_id: String,
    pub lock_family: String,
    pub db_path: PathBuf,
    pub required_fields: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ContaminantDbMaterializationReport {
    pub schema_version: String,
    pub materialization_root: PathBuf,
    pub bundles: Vec<MaterializedDbBundle>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TaxonomyDbMaterializationReport {
    pub schema_version: String,
    pub bundle_id: String,
    pub lock_family: String,
    pub db_path: PathBuf,
    pub required_fields: Vec<String>,
    pub advisory_only: bool,
}
