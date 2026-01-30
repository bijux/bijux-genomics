use std::collections::BTreeSet;
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::Digest;

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
pub fn polyx_bank_path() -> std::path::PathBuf {
    std::path::PathBuf::from("assets/polyx/bank.v1.yaml")
}

#[must_use]
pub fn polyx_presets_path() -> std::path::PathBuf {
    std::path::PathBuf::from("assets/polyx/presets.v1.yaml")
}

/// Load the polyX bank YAML and validate its contents.
///
/// # Errors
/// Returns an error if the file cannot be read, parsed, or fails validation.
pub fn load_polyx_bank(path: &Path) -> Result<PolyxBankV1> {
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("read polyx bank {}", path.display()))?;
    let bank: PolyxBankV1 = serde_yaml::from_str(&contents).context("parse polyx bank yaml")?;
    validate_polyx_bank(&bank)?;
    Ok(bank)
}

/// Load polyX presets and validate references against the bank.
///
/// # Errors
/// Returns an error if the file cannot be read, parsed, or fails validation.
pub fn load_polyx_presets(path: &Path, bank: &PolyxBankV1) -> Result<PolyxPresetsV1> {
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("read polyx presets {}", path.display()))?;
    let presets: PolyxPresetsV1 =
        serde_yaml::from_str(&contents).context("parse polyx presets yaml")?;
    validate_polyx_presets(&presets, bank)?;
    Ok(presets)
}

/// Resolve a preset into an effective polyX set.
///
/// # Errors
/// Returns an error if the preset name is unknown or references missing polyX ids.
pub fn resolve_polyx_preset(
    bank: &PolyxBankV1,
    presets: &PolyxPresetsV1,
    preset_name: &str,
) -> Result<EffectivePolyxSet> {
    let preset = presets
        .presets
        .iter()
        .find(|preset| preset.name == preset_name)
        .ok_or_else(|| anyhow!("unknown polyx preset {preset_name}"))?;

    let (enabled_ids, entries, sequences) = resolve_polyx_ids_and_sequences(bank, preset)?;
    let actual_hash = hash_preset_sequences(&sequences);
    if actual_hash != preset.hash {
        return Err(anyhow!(
            "preset {} hash mismatch (expected {}, got {})",
            preset.name,
            preset.hash,
            actual_hash
        ));
    }

    Ok(EffectivePolyxSet {
        preset: preset.name.clone(),
        preset_hash: preset.hash.clone(),
        rationale: preset.rationale.clone(),
        references: preset.references.clone(),
        notes: preset.notes.clone(),
        sequences,
        enabled_ids,
        entries,
    })
}

fn validate_polyx_bank(bank: &PolyxBankV1) -> Result<()> {
    if bank.entries.is_empty() {
        return Err(anyhow!("polyx bank contains no entries"));
    }
    if bank.bank_id.trim().is_empty() {
        return Err(anyhow!("polyx bank missing bank_id"));
    }
    if bank.version.trim().is_empty() {
        return Err(anyhow!("polyx bank missing version"));
    }
    let mut ids = BTreeSet::new();
    for entry in &bank.entries {
        if !ids.insert(entry.id.clone()) {
            return Err(anyhow!("duplicate polyx id {}", entry.id));
        }
        ensure_sequence_alphabet(&entry.sequence)?;
    }
    Ok(())
}

fn validate_polyx_presets(presets: &PolyxPresetsV1, bank: &PolyxBankV1) -> Result<()> {
    for preset in &presets.presets {
        if preset.name.trim().is_empty() {
            return Err(anyhow!("polyx preset missing name"));
        }
        for polyx_id in &preset.polyx_ids {
            if !bank.entries.iter().any(|entry| entry.id == *polyx_id) {
                return Err(anyhow!("unknown polyx id {polyx_id}"));
            }
        }
        for seq in &preset.sequences {
            ensure_sequence_alphabet(seq)?;
        }
    }
    Ok(())
}

fn resolve_polyx_ids_and_sequences(
    bank: &PolyxBankV1,
    preset: &PolyxPresetV1,
) -> Result<(Vec<String>, Vec<PolyxEntryV1>, Vec<String>)> {
    let mut selected: BTreeSet<String> = BTreeSet::new();
    for entry in &bank.entries {
        if entry.enabled_by_default {
            selected.insert(entry.id.clone());
        }
    }
    for polyx_id in &preset.polyx_ids {
        selected.insert(polyx_id.clone());
    }

    let mut enabled_ids: Vec<String> = selected.into_iter().collect();
    enabled_ids.sort();
    let mut entries = Vec::new();
    for polyx_id in &enabled_ids {
        let entry = bank
            .entries
            .iter()
            .find(|entry| entry.id == *polyx_id)
            .ok_or_else(|| anyhow!("missing polyx id {polyx_id}"))?;
        entries.push(entry.clone());
    }
    let mut sequences: Vec<String> = entries.iter().map(|entry| entry.sequence.clone()).collect();
    sequences.extend(preset.sequences.clone());
    Ok((enabled_ids, entries, sequences))
}

fn ensure_sequence_alphabet(sequence: &str) -> Result<()> {
    if sequence.trim().is_empty() {
        return Err(anyhow!("sequence cannot be empty"));
    }
    for ch in sequence.chars() {
        match ch.to_ascii_uppercase() {
            'A' | 'C' | 'G' | 'T' | 'N' => {}
            _ => return Err(anyhow!("invalid base {ch} in sequence")),
        }
    }
    Ok(())
}

fn hash_preset_sequences(sequences: &[String]) -> String {
    let mut hasher = sha2::Sha256::new();
    for seq in sequences {
        hasher.update(seq.as_bytes());
        hasher.update(b"|");
    }
    format!("{:x}", hasher.finalize())
}
