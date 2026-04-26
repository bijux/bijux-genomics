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
        if motif.id.trim().is_empty() {
            return Err(anyhow!("contaminant motif missing id"));
        }
        if !ids.insert(motif.id.clone()) {
            return Err(anyhow!("duplicate contaminant motif id {}", motif.id));
        }
        if motif.name.trim().is_empty()
            || motif.rationale.trim().is_empty()
            || motif.source.trim().is_empty()
        {
            return Err(anyhow!(
                "contaminant motif {} missing name/rationale/source",
                motif.id
            ));
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::banks::contaminant::ContaminantMotifEntryV1;

    fn valid_motif() -> ContaminantMotifEntryV1 {
        ContaminantMotifEntryV1 {
            id: "phi-x".to_string(),
            name: "PhiX motif".to_string(),
            sequence: "ACGT".to_string(),
            enabled_by_default: true,
            rationale: "sequencing control motif".to_string(),
            source: "NCBI".to_string(),
            notes: String::new(),
        }
    }

    fn motif_bank_with(motif: ContaminantMotifEntryV1) -> ContaminantMotifBankV1 {
        ContaminantMotifBankV1 {
            schema_version: "bijux.fastq.contaminants.v1".to_string(),
            bank_id: "contaminant-bank".to_string(),
            version: "2026-01-01".to_string(),
            motifs: vec![motif],
        }
    }

    #[test]
    fn contaminant_motif_bank_rejects_blank_motif_id() {
        let mut motif = valid_motif();
        motif.id = " ".to_string();

        let err = validate_contaminant_motifs(&motif_bank_with(motif))
            .expect_err("blank contaminant motif ids must be invalid");

        assert!(err.to_string().contains("missing id"));
    }

    #[test]
    fn contaminant_motif_bank_rejects_incomplete_motif_metadata() {
        let mut motif = valid_motif();
        motif.source.clear();

        let err = validate_contaminant_motifs(&motif_bank_with(motif))
            .expect_err("contaminant motif provenance must be complete");

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
