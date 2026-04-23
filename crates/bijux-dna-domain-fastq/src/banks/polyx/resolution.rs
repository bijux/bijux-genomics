use std::collections::BTreeSet;

use anyhow::{anyhow, Result};
use sha2::Digest;

use super::models::{EffectivePolyxSet, PolyxBankV1, PolyxEntryV1, PolyxPresetV1, PolyxPresetsV1};

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

fn hash_preset_sequences(sequences: &[String]) -> String {
    let mut hasher = sha2::Sha256::new();
    for seq in sequences {
        hasher.update(seq.as_bytes());
        hasher.update(b"|");
    }
    hasher.finalize().iter().map(|byte| format!("{byte:02x}")).collect()
}
