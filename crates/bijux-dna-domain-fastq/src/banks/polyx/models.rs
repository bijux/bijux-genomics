use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolyxBankV1 {
    pub schema_version: String,
    pub bank_id: String,
    pub version: String,
    pub entries: Vec<PolyxEntryV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolyxEntryV1 {
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
pub struct PolyxPresetsV1 {
    pub schema_version: String,
    pub presets: Vec<PolyxPresetV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolyxPresetV1 {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub polyx_ids: Vec<String>,
    #[serde(default)]
    pub sequences: Vec<String>,
    pub rationale: String,
    #[serde(default)]
    pub references: Vec<String>,
    #[serde(default)]
    pub notes: Vec<String>,
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectivePolyxSet {
    pub preset: String,
    pub preset_hash: String,
    pub rationale: String,
    pub references: Vec<String>,
    pub notes: Vec<String>,
    pub sequences: Vec<String>,
    pub enabled_ids: Vec<String>,
    pub entries: Vec<PolyxEntryV1>,
}

#[must_use]
pub fn polyx_bank_path() -> PathBuf {
    PathBuf::from("assets/reference/polyx/bank.v1.yaml")
}

#[must_use]
pub fn polyx_presets_path() -> PathBuf {
    PathBuf::from("assets/reference/polyx/presets.v1.yaml")
}
