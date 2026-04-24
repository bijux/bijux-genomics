use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use sha2::Digest;

use super::models::{
    ContaminantMotifBankV1, ContaminantMotifEntryV1, ContaminantPresetV1, ContaminantPresetsV1,
    ContaminantReferenceSpecV1, EffectiveContaminantSet,
};

/// Resolve a preset into an effective contaminant set.
///
/// # Errors
/// Returns an error if the preset name is unknown or references missing motifs.
pub fn resolve_contaminant_preset(
    bank: &ContaminantMotifBankV1,
    presets: &ContaminantPresetsV1,
    preset_name: &str,
    references_dir: &Path,
) -> Result<EffectiveContaminantSet> {
    let preset = presets
        .presets
        .iter()
        .find(|preset| preset.name == preset_name)
        .ok_or_else(|| anyhow!("unknown contaminant preset {preset_name}"))?;

    let (enabled_ids, motifs) = resolve_contaminant_ids(bank, preset)?;
    let actual_hash = hash_preset_contents(&motifs, &preset.references, references_dir)?;
    if actual_hash != preset.hash {
        return Err(anyhow!(
            "preset {} hash mismatch (expected {}, got {})",
            preset.name,
            preset.hash,
            actual_hash
        ));
    }

    Ok(EffectiveContaminantSet {
        preset: preset.name.clone(),
        preset_hash: preset.hash.clone(),
        rationale: preset.rationale.clone(),
        notes: preset.notes.clone(),
        motifs,
        enabled_ids,
        references: preset.references.clone(),
    })
}

fn resolve_contaminant_ids(
    bank: &ContaminantMotifBankV1,
    preset: &ContaminantPresetV1,
) -> Result<(Vec<String>, Vec<ContaminantMotifEntryV1>)> {
    let mut selected: BTreeSet<String> = BTreeSet::new();
    for motif in &bank.motifs {
        if motif.enabled_by_default {
            selected.insert(motif.id.clone());
        }
    }
    for motif_id in &preset.motif_ids {
        selected.insert(motif_id.clone());
    }
    let mut enabled_ids: Vec<String> = selected.into_iter().collect();
    enabled_ids.sort();
    let mut motifs = Vec::new();
    for motif_id in &enabled_ids {
        let motif = bank
            .motifs
            .iter()
            .find(|entry| entry.id == *motif_id)
            .ok_or_else(|| anyhow!("missing contaminant motif id {motif_id}"))?;
        motifs.push(motif.clone());
    }
    Ok((enabled_ids, motifs))
}

fn hash_preset_contents(
    motifs: &[ContaminantMotifEntryV1],
    references: &[ContaminantReferenceSpecV1],
    references_dir: &Path,
) -> Result<String> {
    let mut hasher = sha2::Sha256::new();
    for motif in motifs {
        hasher.update(motif.sequence.as_bytes());
        hasher.update(b"|");
    }
    for reference in references {
        let path = references_dir.join(&reference.file);
        let contents = std::fs::read(&path).with_context(|| format!("read {}", path.display()))?;
        hasher.update(&contents);
        hasher.update(b"|");
    }
    Ok(sha256_hex(hasher.finalize()))
}

fn sha256_hex(digest: impl AsRef<[u8]>) -> String {
    let bytes = digest.as_ref();
    let mut hex = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(&mut hex, "{byte:02x}");
    }
    hex
}
