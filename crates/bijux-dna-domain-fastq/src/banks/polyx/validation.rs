use std::collections::BTreeSet;

use anyhow::{anyhow, Result};

use super::models::{PolyxBankV1, PolyxPresetsV1};

pub(super) fn validate_polyx_bank(bank: &PolyxBankV1) -> Result<()> {
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

pub(super) fn validate_polyx_presets(presets: &PolyxPresetsV1, bank: &PolyxBankV1) -> Result<()> {
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
