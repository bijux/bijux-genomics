use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::Digest;

const ADAPTER_TAGS: [&str; 9] = [
    "truseq", "nextera", "ssdna", "umi", "pcr", "custom", "nebnext", "capture", "partial",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdapterBankV1 {
    pub schema_version: String,
    pub bank_id: String,
    pub version: String,
    pub provenance_status: String,
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
    pub tags: Vec<String>,
    #[serde(default)]
    pub adapter_ids: Vec<String>,
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
pub struct EffectiveAdapterSet {
    pub preset: String,
    pub preset_hash: String,
    pub preset_tags: Vec<String>,
    pub rationale: String,
    pub references: Vec<String>,
    pub notes: Vec<String>,
    pub sequences: Vec<String>,
    pub enabled_ids: Vec<String>,
    pub adapters: Vec<AdapterEntryV1>,
}

#[must_use]
pub fn adapter_bank_path() -> std::path::PathBuf {
    std::path::PathBuf::from("assets/adapters/bank.v1.yaml")
}

#[must_use]
pub fn adapter_presets_path() -> std::path::PathBuf {
    std::path::PathBuf::from("assets/adapters/presets.v1.yaml")
}

/// Load the adapter bank YAML and validate its contents.
///
/// # Errors
/// Returns an error if the file cannot be read, parsed, or fails validation.
pub fn load_adapter_bank(path: &Path) -> Result<AdapterBankV1> {
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("read adapter bank {}", path.display()))?;
    let bank: AdapterBankV1 =
        bijux_dna_infra::formats::parse_yaml(&contents).context("parse adapter bank yaml")?;
    validate_adapter_bank(&bank)?;
    Ok(bank)
}

/// Load adapter presets and validate references against the bank.
///
/// # Errors
/// Returns an error if the file cannot be read, parsed, or fails validation.
pub fn load_adapter_presets(path: &Path, bank: &AdapterBankV1) -> Result<AdapterPresetsV1> {
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("read adapter presets {}", path.display()))?;
    let presets: AdapterPresetsV1 =
        bijux_dna_infra::formats::parse_yaml(&contents).context("parse adapter presets yaml")?;
    validate_adapter_presets(&presets, bank)?;
    Ok(presets)
}

/// Resolve a preset into an effective adapter set with explicit overrides.
///
/// # Errors
/// Returns an error if the preset name is unknown or overrides reference missing adapters.
pub fn resolve_adapter_preset(
    bank: &AdapterBankV1,
    presets: &AdapterPresetsV1,
    preset_name: &str,
    enable: &[String],
    disable: &[String],
) -> Result<EffectiveAdapterSet> {
    let preset = presets
        .presets
        .iter()
        .find(|preset| preset.name == preset_name)
        .ok_or_else(|| anyhow!("unknown adapter preset {preset_name}"))?;

    let (enabled_ids, adapters, sequences) =
        resolve_adapter_ids_and_sequences(bank, preset, enable, disable)?;
    let actual_hash = hash_preset_sequences(&sequences);
    if actual_hash != preset.hash {
        return Err(anyhow!(
            "preset {} hash mismatch (expected {}, got {})",
            preset.name,
            preset.hash,
            actual_hash
        ));
    }

    Ok(EffectiveAdapterSet {
        preset: preset.name.clone(),
        preset_hash: preset.hash.clone(),
        preset_tags: preset.tags.clone(),
        rationale: preset.rationale.clone(),
        references: preset.references.clone(),
        notes: preset.notes.clone(),
        sequences,
        enabled_ids,
        adapters,
    })
}

fn validate_adapter_bank(bank: &AdapterBankV1) -> Result<()> {
    if bank.adapters.is_empty() {
        return Err(anyhow!("adapter bank contains no entries"));
    }
    if bank.bank_id.trim().is_empty() {
        return Err(anyhow!("adapter bank missing bank_id"));
    }
    if bank.version.trim().is_empty() {
        return Err(anyhow!("adapter bank missing version"));
    }
    if bank.provenance_status != "complete" {
        return Err(anyhow!(
            "adapter bank provenance_status must be `complete` for supported scope"
        ));
    }
    let mut ids = BTreeSet::new();
    for adapter in &bank.adapters {
        if !ids.insert(adapter.id.clone()) {
            return Err(anyhow!("duplicate adapter id {}", adapter.id));
        }
        if adapter.tags.is_empty() {
            return Err(anyhow!("adapter {} has no tags", adapter.id));
        }
        for tag in &adapter.tags {
            if !ADAPTER_TAGS.contains(&tag.as_str()) {
                return Err(anyhow!("unknown adapter tag {tag}"));
            }
        }
        ensure_sequence_alphabet(&adapter.sequence)?;
    }
    Ok(())
}

fn resolve_adapter_ids_and_sequences(
    bank: &AdapterBankV1,
    preset: &AdapterPresetV1,
    enable: &[String],
    disable: &[String],
) -> Result<(Vec<String>, Vec<AdapterEntryV1>, Vec<String>)> {
    let mut selected: BTreeSet<String> = BTreeSet::new();
    for adapter in &bank.adapters {
        if adapter
            .tags
            .iter()
            .any(|tag| preset.tags.iter().any(|preset_tag| preset_tag == tag))
        {
            selected.insert(adapter.id.clone());
        }
    }
    for adapter_id in &preset.adapter_ids {
        selected.insert(adapter_id.clone());
    }

    for adapter_id in disable {
        selected.remove(adapter_id);
    }
    for adapter_id in enable {
        if !bank
            .adapters
            .iter()
            .any(|adapter| adapter.id == *adapter_id)
        {
            return Err(anyhow!("unknown adapter id {adapter_id}"));
        }
        selected.insert(adapter_id.clone());
    }

    let mut enabled_ids: Vec<String> = selected.into_iter().collect();
    enabled_ids.sort();
    let mut adapters = Vec::new();
    for adapter_id in &enabled_ids {
        let adapter = bank
            .adapters
            .iter()
            .find(|adapter| adapter.id == *adapter_id)
            .ok_or_else(|| anyhow!("missing adapter id {adapter_id}"))?;
        adapters.push(adapter.clone());
    }
    let sequences: Vec<String> = adapters
        .iter()
        .map(|adapter| adapter.sequence.clone())
        .collect();
    Ok((enabled_ids, adapters, sequences))
}

fn validate_adapter_presets(presets: &AdapterPresetsV1, bank: &AdapterBankV1) -> Result<()> {
    let mut names = BTreeSet::new();
    let bank_ids: BTreeSet<String> = bank.adapters.iter().map(|a| a.id.clone()).collect();
    for preset in &presets.presets {
        if !names.insert(preset.name.clone()) {
            return Err(anyhow!("duplicate preset name {}", preset.name));
        }
        if preset.hash.trim().is_empty() {
            return Err(anyhow!("preset {} missing hash", preset.name));
        }
        if preset.rationale.trim().is_empty() {
            return Err(anyhow!("preset {} missing rationale", preset.name));
        }
        if preset.sequences.is_empty() && preset.name != "none" {
            return Err(anyhow!("preset {} missing sequences", preset.name));
        }
        for tag in &preset.tags {
            if !ADAPTER_TAGS.contains(&tag.as_str()) {
                return Err(anyhow!("unknown preset tag {tag}"));
            }
        }
        for adapter_id in &preset.adapter_ids {
            if !bank_ids.contains(adapter_id) {
                return Err(anyhow!(
                    "preset {} references unknown adapter {}",
                    preset.name,
                    adapter_id
                ));
            }
        }
        let (_, _, sequences) = resolve_adapter_ids_and_sequences(bank, preset, &[], &[])?;
        if !preset.sequences.is_empty() || preset.name == "none" {
            let mut expected = sequences.clone();
            expected.sort();
            let mut actual = preset.sequences.clone();
            actual.sort();
            if expected != actual {
                return Err(anyhow!(
                    "preset {} sequences do not match bank selection",
                    preset.name
                ));
            }
        }
        let actual_hash = hash_preset_sequences(&sequences);
        if actual_hash != preset.hash {
            return Err(anyhow!(
                "preset {} hash mismatch (expected {}, got {})",
                preset.name,
                preset.hash,
                actual_hash
            ));
        }
    }
    Ok(())
}

fn hash_preset_sequences(sequences: &[String]) -> String {
    let mut ordered: Vec<String> = sequences.to_vec();
    ordered.sort();
    let joined = ordered.join("|");
    let digest = sha2::Sha256::digest(joined.as_bytes());
    format!("{digest:x}")
}

fn ensure_sequence_alphabet(sequence: &str) -> Result<()> {
    let mut invalid = Vec::new();
    for ch in sequence.chars() {
        let upper = ch.to_ascii_uppercase();
        if !matches!(upper, 'A' | 'C' | 'G' | 'T' | 'N') {
            invalid.push(ch);
        }
    }
    if !invalid.is_empty() {
        return Err(anyhow!(
            "invalid adapter sequence alphabet: {}",
            invalid.into_iter().collect::<String>()
        ));
    }
    Ok(())
}

#[must_use]
pub fn adapter_categories() -> BTreeSet<String> {
    ADAPTER_TAGS.iter().map(|tag| (*tag).to_string()).collect()
}

#[must_use]
pub fn adapters_by_category(bank: &AdapterBankV1) -> BTreeMap<String, Vec<String>> {
    let mut map: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for adapter in &bank.adapters {
        for tag in &adapter.tags {
            map.entry(tag.clone()).or_default().push(adapter.id.clone());
        }
    }
    for ids in map.values_mut() {
        ids.sort();
    }
    map
}
