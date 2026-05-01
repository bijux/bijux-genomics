use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdapterBankV1 {
    pub schema_version: String,
    pub bank_id: String,
    pub version: String,
    pub provenance_status: String,
    pub license: String,
    pub source_document: String,
    pub source_checksum_sha256: String,
    pub applicable_assays: Vec<String>,
    pub selection_logic: String,
    pub adapters: Vec<AdapterEntryV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdapterEntryV1 {
    pub id: String,
    pub tags: Vec<String>,
    pub name: String,
    pub sequence: String,
    pub read_scope: ReadScope,
    pub enabled_by_default: bool,
    pub rationale: String,
    pub source: String,
    pub notes: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReadScope {
    R1,
    R2,
    Both,
    SingleEnd,
    PairedEnd,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdapterPresetsV1 {
    pub schema_version: String,
    pub presets: Vec<AdapterPresetV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdapterPresetV1 {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub applicable_assays: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub adapter_ids: Vec<String>,
    #[serde(default)]
    pub sequences: Vec<String>,
    pub rationale: String,
    pub selection_logic: String,
    #[serde(default)]
    pub references: Vec<String>,
    #[serde(default)]
    pub notes: Vec<String>,
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectiveAdapterSet {
    pub preset: String,
    pub preset_hash: String,
    pub applicable_assays: Vec<String>,
    pub preset_tags: Vec<String>,
    pub rationale: String,
    pub selection_logic: String,
    pub references: Vec<String>,
    pub notes: Vec<String>,
    pub sequences: Vec<String>,
    pub enabled_ids: Vec<String>,
    pub adapters: Vec<AdapterEntryV1>,
}

#[must_use]
pub fn adapter_bank_path() -> std::path::PathBuf {
    std::path::PathBuf::from("assets/reference/adapters/bank.v1.yaml")
}

#[must_use]
pub fn adapter_presets_path() -> std::path::PathBuf {
    std::path::PathBuf::from("assets/reference/adapters/presets.v1.yaml")
}
