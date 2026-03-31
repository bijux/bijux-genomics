use std::collections::{BTreeMap, BTreeSet};

use anyhow::{anyhow, Result};
use sha2::Digest;

use super::{
    AdapterBankV1, AdapterEntryV1, AdapterPresetV1, AdapterPresetsV1, EffectiveAdapterSet,
};

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

pub(super) fn resolve_adapter_ids_and_sequences(
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

pub(super) fn hash_preset_sequences(sequences: &[String]) -> String {
    let mut ordered: Vec<String> = sequences.to_vec();
    ordered.sort();
    let joined = ordered.join("|");
    let digest = sha2::Sha256::digest(joined.as_bytes());
    format!("{digest:x}")
}

#[must_use]
pub fn adapter_categories() -> BTreeSet<String> {
    super::ADAPTER_TAGS
        .iter()
        .map(|tag| (*tag).to_string())
        .collect()
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
