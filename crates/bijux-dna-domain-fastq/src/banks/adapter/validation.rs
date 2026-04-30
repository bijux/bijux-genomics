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
    if bank.license.trim().is_empty() {
        return Err(anyhow!("adapter bank missing license"));
    }
    if bank.source_document.trim().is_empty() {
        return Err(anyhow!("adapter bank missing source_document"));
    }
    if bank.source_checksum_sha256.trim().len() != 64 {
        return Err(anyhow!("adapter bank missing source_checksum_sha256"));
    }
    if bank.applicable_assays.is_empty() {
        return Err(anyhow!("adapter bank missing applicable_assays"));
    }
    if bank.selection_logic.trim().is_empty() {
        return Err(anyhow!("adapter bank missing selection_logic"));
    }
    if bank.provenance_status != "complete" {
        return Err(anyhow!(
            "adapter bank provenance_status must be `complete` for supported scope"
        ));
    }
    let mut ids = BTreeSet::new();
    for adapter in &bank.adapters {
        if adapter.id.trim().is_empty() {
            return Err(anyhow!("adapter entry missing id"));
        }
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
        if preset.name.trim().is_empty() {
            return Err(anyhow!("adapter preset missing name"));
        }
        if !names.insert(preset.name.clone()) {
            return Err(anyhow!("duplicate preset name {}", preset.name));
        }
        if preset.hash.trim().is_empty() {
            return Err(anyhow!("preset {} missing hash", preset.name));
        }
        if preset.rationale.trim().is_empty() {
            return Err(anyhow!("preset {} missing rationale", preset.name));
        }
        if preset.selection_logic.trim().is_empty() {
            return Err(anyhow!("preset {} missing selection_logic", preset.name));
        }
        if preset.applicable_assays.is_empty() && preset.name != "none" {
            return Err(anyhow!("preset {} missing applicable_assays", preset.name));
        }
        if preset.sequences.is_empty() && preset.name != "none" {
            return Err(anyhow!("preset {} missing sequences", preset.name));
        }
        for tag in &preset.tags {
            if !super::ADAPTER_TAGS.contains(&tag.as_str()) {
                return Err(anyhow!("unknown preset tag {tag}"));
            }
        }
        let mut preset_adapter_ids = BTreeSet::new();
        for adapter_id in &preset.adapter_ids {
            if adapter_id.trim().is_empty() {
                return Err(anyhow!("preset {} has empty adapter id", preset.name));
            }
            if !preset_adapter_ids.insert(adapter_id.clone()) {
                return Err(anyhow!("preset {} repeats adapter id {}", preset.name, adapter_id));
            }
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
    if sequence.trim().is_empty() {
        return Err(anyhow!("sequence cannot be empty"));
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::banks::adapter::{AdapterEntryV1, AdapterPresetV1, ReadScope};

    fn valid_bank_with(adapter: AdapterEntryV1) -> AdapterBankV1 {
        AdapterBankV1 {
            schema_version: "bijux.fastq.adapter_bank.v1".to_string(),
            bank_id: "adapter-bank".to_string(),
            version: "2026-01-01".to_string(),
            provenance_status: "complete".to_string(),
            license: "CC-BY-4.0".to_string(),
            source_document: "assets/reference/EVIDENCE.md".to_string(),
            source_checksum_sha256:
                "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string(),
            applicable_assays: vec!["shotgun".to_string()],
            selection_logic: "select explicit preset by assay or library-prep family".to_string(),
            adapters: vec![adapter],
        }
    }

    fn valid_adapter() -> AdapterEntryV1 {
        AdapterEntryV1 {
            id: "illumina-universal".to_string(),
            tags: vec!["truseq".to_string()],
            name: "Illumina universal".to_string(),
            sequence: "AGATCGGAAGAG".to_string(),
            read_scope: ReadScope::Both,
            enabled_by_default: true,
            rationale: "standard Illumina adapter".to_string(),
            source: "vendor".to_string(),
            notes: String::new(),
        }
    }

    fn assert_err<T>(result: Result<T>, message: &str) -> anyhow::Error {
        match result {
            Ok(_) => panic!("{message}"),
            Err(err) => err,
        }
    }

    #[test]
    fn adapter_bank_rejects_blank_adapter_id() {
        let mut adapter = valid_adapter();
        adapter.id = " ".to_string();

        let err = assert_err(
            validate_adapter_bank(&valid_bank_with(adapter)),
            "blank adapter ids must be invalid",
        );

        assert!(err.to_string().contains("missing id"));
    }

    #[test]
    fn adapter_bank_rejects_blank_adapter_sequence() {
        let mut adapter = valid_adapter();
        adapter.sequence = " ".to_string();

        let err = assert_err(
            validate_adapter_bank(&valid_bank_with(adapter)),
            "blank adapter sequences must be invalid",
        );

        assert!(err.to_string().contains("sequence cannot be empty"));
    }

    fn valid_presets_with(preset: AdapterPresetV1) -> AdapterPresetsV1 {
        AdapterPresetsV1 {
            schema_version: "bijux.fastq.adapter_presets.v1".to_string(),
            presets: vec![preset],
        }
    }

    fn valid_preset() -> AdapterPresetV1 {
        let sequences = vec!["AGATCGGAAGAG".to_string()];
        AdapterPresetV1 {
            name: "truseq".to_string(),
            description: None,
            applicable_assays: vec!["shotgun".to_string()],
            tags: vec!["truseq".to_string()],
            adapter_ids: Vec::new(),
            sequences: sequences.clone(),
            rationale: "default Illumina preset".to_string(),
            selection_logic: "select when assay uses standard Illumina shotgun library prep"
                .to_string(),
            references: Vec::new(),
            notes: Vec::new(),
            hash: hash_preset_sequences(&sequences),
        }
    }

    #[test]
    fn adapter_presets_reject_blank_name() {
        let bank = valid_bank_with(valid_adapter());
        let mut preset = valid_preset();
        preset.name = " ".to_string();

        let err = assert_err(
            validate_adapter_presets(&valid_presets_with(preset), &bank),
            "blank adapter preset names must be invalid",
        );

        assert!(err.to_string().contains("missing name"));
    }

    #[test]
    fn adapter_presets_reject_repeated_adapter_ids() {
        let bank = valid_bank_with(valid_adapter());
        let mut preset = valid_preset();
        preset.tags.clear();
        preset.adapter_ids =
            vec!["illumina-universal".to_string(), "illumina-universal".to_string()];

        let err = assert_err(
            validate_adapter_presets(&valid_presets_with(preset), &bank),
            "repeated adapter ids must be invalid",
        );

        assert!(err.to_string().contains("repeats adapter id"));
    }

    #[test]
    fn adapter_bank_rejects_missing_license() {
        let mut bank = valid_bank_with(valid_adapter());
        bank.license.clear();

        let err = assert_err(validate_adapter_bank(&bank), "license is required");

        assert!(err.to_string().contains("missing license"));
    }
}
