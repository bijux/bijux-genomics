use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

const ADAPTER_CATEGORIES: [&str; 8] = [
    "truseq",
    "nextera",
    "ssdna_splint",
    "pcr_primers",
    "umi_constructs",
    "kit_custom",
    "capture_linkers",
    "partial_motifs",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdapterBankV1 {
    pub schema_version: String,
    pub adapters: Vec<AdapterEntryV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdapterEntryV1 {
    pub id: String,
    pub category: String,
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
    pub categories: Vec<String>,
    #[serde(default)]
    pub adapter_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectiveAdapterSet {
    pub preset: String,
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
    let bank: AdapterBankV1 = serde_yaml::from_str(&contents).context("parse adapter bank yaml")?;
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
        serde_yaml::from_str(&contents).context("parse adapter presets yaml")?;
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

    let mut selected: BTreeSet<String> = BTreeSet::new();
    for adapter in &bank.adapters {
        if preset.categories.iter().any(|c| c == &adapter.category) {
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

    Ok(EffectiveAdapterSet {
        preset: preset.name.clone(),
        enabled_ids,
        adapters,
    })
}

fn validate_adapter_bank(bank: &AdapterBankV1) -> Result<()> {
    if bank.adapters.is_empty() {
        return Err(anyhow!("adapter bank contains no entries"));
    }
    let mut ids = BTreeSet::new();
    for adapter in &bank.adapters {
        if !ids.insert(adapter.id.clone()) {
            return Err(anyhow!("duplicate adapter id {}", adapter.id));
        }
        if !ADAPTER_CATEGORIES.contains(&adapter.category.as_str()) {
            return Err(anyhow!("unknown adapter category {}", adapter.category));
        }
        ensure_sequence_alphabet(&adapter.sequence)?;
    }
    Ok(())
}

fn validate_adapter_presets(presets: &AdapterPresetsV1, bank: &AdapterBankV1) -> Result<()> {
    let mut names = BTreeSet::new();
    let bank_ids: BTreeSet<String> = bank.adapters.iter().map(|a| a.id.clone()).collect();
    for preset in &presets.presets {
        if !names.insert(preset.name.clone()) {
            return Err(anyhow!("duplicate preset name {}", preset.name));
        }
        for category in &preset.categories {
            if !ADAPTER_CATEGORIES.contains(&category.as_str()) {
                return Err(anyhow!("unknown preset category {category}"));
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
    }
    Ok(())
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
    ADAPTER_CATEGORIES
        .iter()
        .map(|c| (*c).to_string())
        .collect()
}

#[must_use]
pub fn adapters_by_category(bank: &AdapterBankV1) -> BTreeMap<String, Vec<String>> {
    let mut map: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for adapter in &bank.adapters {
        map.entry(adapter.category.clone())
            .or_default()
            .push(adapter.id.clone());
    }
    for ids in map.values_mut() {
        ids.sort();
    }
    map
}
