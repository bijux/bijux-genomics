use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContaminantMotifBankV1 {
    pub schema_version: String,
    pub bank_id: String,
    pub version: String,
    pub motifs: Vec<ContaminantMotifEntryV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContaminantMotifEntryV1 {
    pub id: String,
    pub name: String,
    pub sequence: String,
    pub enabled_by_default: bool,
    pub rationale: String,
    pub source: String,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContaminantPresetsV1 {
    pub schema_version: String,
    pub presets: Vec<ContaminantPresetV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContaminantReferenceSpecV1 {
    pub id: String,
    pub file: String,
    pub rationale: String,
    pub source: String,
    #[serde(default)]
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContaminantPresetV1 {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub motif_ids: Vec<String>,
    #[serde(default)]
    pub references: Vec<ContaminantReferenceSpecV1>,
    pub rationale: String,
    #[serde(default)]
    pub notes: Vec<String>,
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectiveContaminantSet {
    pub preset: String,
    pub preset_hash: String,
    pub rationale: String,
    pub notes: Vec<String>,
    pub motifs: Vec<ContaminantMotifEntryV1>,
    pub enabled_ids: Vec<String>,
    pub references: Vec<ContaminantReferenceSpecV1>,
}

#[must_use]
pub fn contaminant_motifs_path() -> PathBuf {
    PathBuf::from("assets/reference/contaminants/contaminant_motifs.v1.yaml")
}

#[must_use]
pub fn contaminant_presets_path() -> PathBuf {
    PathBuf::from("assets/reference/contaminants/presets.v1.yaml")
}

#[must_use]
pub fn contaminant_references_dir() -> PathBuf {
    PathBuf::from("assets/reference/contaminants/references")
}
