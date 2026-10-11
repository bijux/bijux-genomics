use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReferenceBankEntry {
    pub species_id: String,
    pub build_id: String,
    pub source_accession: String,
    pub fasta_url: String,
    pub fasta_md5: String,
    pub fasta_sha256: String,
    pub fasta_size_bytes: u64,
    pub license_id: String,
    pub license_url: String,
    #[serde(default)]
    pub required_indexes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferenceSourceLock {
    pub schema_version: u64,
    pub source: String,
    pub source_evidence_commit: String,
    pub references: Vec<ReferenceSourceLockEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferenceSourceLockEntry {
    pub species_id: String,
    pub build_id: String,
    pub source_accession: String,
    pub fasta_url: String,
    pub fasta_md5: String,
    pub fasta_sha256: String,
    pub size_bytes: u64,
    pub license_id: String,
    pub license_url: String,
    pub evidence_uri: String,
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

/// Metadata identity projected from a complete reference bundle declaration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReferenceBundleIdentity {
    pub bundle_id: String,
    pub species_id: String,
    pub build_id: String,
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
