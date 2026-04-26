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
        if entry.id.trim().is_empty() {
            return Err(anyhow!("polyx entry missing id"));
        }
        if !ids.insert(entry.id.clone()) {
            return Err(anyhow!("duplicate polyx id {}", entry.id));
        }
        if entry.name.trim().is_empty()
            || entry.rationale.trim().is_empty()
            || entry.source.trim().is_empty()
        {
            return Err(anyhow!(
                "polyx entry {} missing name/rationale/source",
                entry.id
            ));
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::banks::polyx::PolyxEntryV1;

    fn valid_entry() -> PolyxEntryV1 {
        PolyxEntryV1 {
            id: "poly-g".to_string(),
            name: "Poly-G tail".to_string(),
            sequence: "GGGGGG".to_string(),
            enabled_by_default: true,
            rationale: "two-color chemistry artifact".to_string(),
            source: "Illumina".to_string(),
            notes: String::new(),
        }
    }

    fn bank_with(entry: PolyxEntryV1) -> PolyxBankV1 {
        PolyxBankV1 {
            schema_version: "bijux.fastq.polyx_bank.v1".to_string(),
            bank_id: "polyx-bank".to_string(),
            version: "2026-01-01".to_string(),
            entries: vec![entry],
        }
    }

    #[test]
    fn polyx_bank_rejects_blank_entry_id() {
        let mut entry = valid_entry();
        entry.id = " ".to_string();

        let err = validate_polyx_bank(&bank_with(entry))
            .expect_err("blank polyx ids must be invalid");

        assert!(err.to_string().contains("missing id"));
    }

    #[test]
    fn polyx_bank_rejects_incomplete_entry_metadata() {
        let mut entry = valid_entry();
        entry.rationale.clear();

        let err = validate_polyx_bank(&bank_with(entry))
            .expect_err("polyx metadata must be complete");

        assert!(err.to_string().contains("missing name/rationale/source"));
    }
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
