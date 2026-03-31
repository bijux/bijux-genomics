use std::collections::BTreeSet;

use anyhow::{anyhow, Result};

use super::{AdapterBankV1, AdapterPresetsV1};
use crate::banks::adapter::resolution::{hash_preset_sequences, resolve_adapter_ids_and_sequences};

pub(super) fn validate_adapter_bank(bank: &AdapterBankV1) -> Result<()> {
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
            if !super::ADAPTER_TAGS.contains(&tag.as_str()) {
                return Err(anyhow!("unknown adapter tag {tag}"));
            }
        }
        ensure_sequence_alphabet(&adapter.sequence)?;
    }
    Ok(())
}

pub(super) fn validate_adapter_presets(
    presets: &AdapterPresetsV1,
    bank: &AdapterBankV1,
) -> Result<()> {
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
            if !super::ADAPTER_TAGS.contains(&tag.as_str()) {
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
