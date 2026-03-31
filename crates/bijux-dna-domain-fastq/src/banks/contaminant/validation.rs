use std::collections::BTreeSet;
use std::path::Path;

use anyhow::{anyhow, Result};

use super::models::{ContaminantMotifBankV1, ContaminantPresetsV1};

pub(super) fn validate_contaminant_motifs(bank: &ContaminantMotifBankV1) -> Result<()> {
    if bank.motifs.is_empty() {
        return Err(anyhow!("contaminant motif bank contains no entries"));
    }
    if bank.bank_id.trim().is_empty() {
        return Err(anyhow!("contaminant motif bank missing bank_id"));
    }
    if bank.version.trim().is_empty() {
        return Err(anyhow!("contaminant motif bank missing version"));
    }
    let mut ids = BTreeSet::new();
    for motif in &bank.motifs {
        if !ids.insert(motif.id.clone()) {
            return Err(anyhow!("duplicate contaminant motif id {}", motif.id));
        }
        ensure_sequence_alphabet(&motif.sequence)?;
    }
    Ok(())
}

pub(super) fn validate_contaminant_presets(
    presets: &ContaminantPresetsV1,
    bank: &ContaminantMotifBankV1,
    references_dir: &Path,
) -> Result<()> {
    for preset in &presets.presets {
        if preset.name.trim().is_empty() {
            return Err(anyhow!("contaminant preset missing name"));
        }
        for motif_id in &preset.motif_ids {
            if !bank.motifs.iter().any(|motif| motif.id == *motif_id) {
                return Err(anyhow!("unknown contaminant motif id {motif_id}"));
            }
        }
        for reference in &preset.references {
            if reference.id.trim().is_empty() {
                return Err(anyhow!("contaminant reference missing id"));
            }
            if reference.file.trim().is_empty() {
                return Err(anyhow!("contaminant reference missing file"));
            }
            let path = references_dir.join(&reference.file);
            if !path.exists() {
                return Err(anyhow!("missing contaminant reference {}", path.display()));
            }
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
