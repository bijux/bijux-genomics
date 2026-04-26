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
            return Err(anyhow!("polyx entry {} missing name/rationale/source", entry.id));
        }
        ensure_sequence_alphabet(&entry.sequence)?;
    }
    Ok(())
}

pub(super) fn validate_polyx_presets(presets: &PolyxPresetsV1, bank: &PolyxBankV1) -> Result<()> {
    let mut names = BTreeSet::new();
    for preset in &presets.presets {
        if preset.name.trim().is_empty() {
            return Err(anyhow!("polyx preset missing name"));
        }
        if !names.insert(preset.name.clone()) {
            return Err(anyhow!("duplicate polyx preset name {}", preset.name));
        }
        if preset.hash.trim().is_empty() {
            return Err(anyhow!("polyx preset {} missing hash", preset.name));
        }
        if preset.rationale.trim().is_empty() {
            return Err(anyhow!("polyx preset {} missing rationale", preset.name));
        }
        let mut polyx_ids = BTreeSet::new();
        for polyx_id in &preset.polyx_ids {
            if polyx_id.trim().is_empty() {
                return Err(anyhow!("polyx preset {} has empty polyx id", preset.name));
            }
            if !polyx_ids.insert(polyx_id.clone()) {
                return Err(anyhow!("polyx preset {} repeats polyx id {}", preset.name, polyx_id));
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::banks::polyx::{PolyxEntryV1, PolyxPresetV1};

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

    fn presets_with(preset: PolyxPresetV1) -> PolyxPresetsV1 {
        PolyxPresetsV1 {
            schema_version: "bijux.fastq.polyx_presets.v1".to_string(),
            presets: vec![preset],
        }
    }

    fn valid_preset() -> PolyxPresetV1 {
        PolyxPresetV1 {
            name: "two-color".to_string(),
            description: None,
            polyx_ids: vec!["poly-g".to_string()],
            sequences: Vec::new(),
            rationale: "default two-color trimming".to_string(),
            references: Vec::new(),
            notes: Vec::new(),
            hash: "sha256:fixture".to_string(),
        }
    }

    fn assert_err<T>(result: Result<T>, message: &str) -> anyhow::Error {
        match result {
            Ok(_) => panic!("{message}"),
            Err(err) => err,
        }
    }

    #[test]
    fn polyx_bank_rejects_blank_entry_id() {
        let mut entry = valid_entry();
        entry.id = " ".to_string();

        let err =
            assert_err(validate_polyx_bank(&bank_with(entry)), "blank polyx ids must be invalid");

        assert!(err.to_string().contains("missing id"));
    }

    #[test]
    fn polyx_bank_rejects_incomplete_entry_metadata() {
        let mut entry = valid_entry();
        entry.rationale.clear();

        let err =
            assert_err(validate_polyx_bank(&bank_with(entry)), "polyx metadata must be complete");

        assert!(err.to_string().contains("missing name/rationale/source"));
    }

    #[test]
    fn polyx_presets_reject_duplicate_names() {
        let bank = bank_with(valid_entry());
        let preset = valid_preset();
        let presets = PolyxPresetsV1 {
            schema_version: "bijux.fastq.polyx_presets.v1".to_string(),
            presets: vec![preset.clone(), preset],
        };

        let err = assert_err(
            validate_polyx_presets(&presets, &bank),
            "duplicate polyx preset names must be invalid",
        );

        assert!(err.to_string().contains("duplicate polyx preset name"));
    }

    #[test]
    fn polyx_presets_reject_repeated_polyx_ids() {
        let bank = bank_with(valid_entry());
        let mut preset = valid_preset();
        preset.polyx_ids = vec!["poly-g".to_string(), "poly-g".to_string()];

        let err = assert_err(
            validate_polyx_presets(&presets_with(preset), &bank),
            "repeated polyx ids must be invalid",
        );

        assert!(err.to_string().contains("repeats polyx id"));
    }
}
