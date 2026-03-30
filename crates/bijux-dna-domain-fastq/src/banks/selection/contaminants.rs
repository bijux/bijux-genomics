use anyhow::{anyhow, Result};

pub const DEFAULT_CONTAMINANT_PRESET: &str = "illumina_default";

pub struct ContaminantSelection {
    pub motifs: crate::ContaminantMotifBankV1,
    pub presets: crate::ContaminantPresetsV1,
    pub preset_name: String,
    pub motifs_checksum: String,
    pub presets_checksum: String,
}

/// Resolve contaminant selection from CLI options.
///
/// # Errors
/// Returns an error if contaminant configs cannot be loaded or the preset is invalid.
pub fn resolve_contaminant_selection(preset: Option<&str>) -> Result<ContaminantSelection> {
    let preset_name = match preset {
        Some(name) if !name.trim().is_empty() => name.trim().to_string(),
        Some(_) => return Err(anyhow!("contaminant preset name is empty")),
        None => DEFAULT_CONTAMINANT_PRESET.to_string(),
    };
    let motifs_path = crate::contaminant_motifs_path();
    let presets_path = crate::contaminant_presets_path();
    let references_dir = crate::contaminant_references_dir();
    let motifs = crate::load_contaminant_motifs(&motifs_path)?;
    let presets = crate::load_contaminant_presets(&presets_path, &motifs, &references_dir)?;
    let motifs_checksum = bijux_dna_infra::hash_file_sha256(&motifs_path)?;
    let presets_checksum = bijux_dna_infra::hash_file_sha256(&presets_path)?;
    Ok(ContaminantSelection {
        motifs,
        presets,
        preset_name,
        motifs_checksum,
        presets_checksum,
    })
}

/// Resolve the effective contaminant set from a selection.
///
/// # Errors
/// Returns an error if the preset is invalid.
pub fn resolve_effective_contaminants(
    selection: &ContaminantSelection,
) -> Result<crate::EffectiveContaminantSet> {
    let references_dir = crate::contaminant_references_dir();
    crate::resolve_contaminant_preset(
        &selection.motifs,
        &selection.presets,
        &selection.preset_name,
        &references_dir,
    )
}

#[must_use]
pub fn contaminant_bank_provenance_json(
    selection: &ContaminantSelection,
    effective: &crate::EffectiveContaminantSet,
) -> serde_json::Value {
    let enabled_entries: Vec<serde_json::Value> = effective
        .motifs
        .iter()
        .map(|entry| {
            serde_json::json!({
                "id": entry.id,
                "sequence": entry.sequence,
                "rationale": entry.rationale,
                "source": entry.source,
            })
        })
        .collect();
    let references_dir = crate::contaminant_references_dir();
    let references: Vec<serde_json::Value> = effective
        .references
        .iter()
        .map(|reference| {
            let path = references_dir.join(&reference.file);
            let fasta = std::fs::read_to_string(&path).unwrap_or_default();
            let hash =
                bijux_dna_infra::hash_file_sha256(&path).unwrap_or_else(|_| "unknown".to_string());
            serde_json::json!({
                "id": reference.id,
                "file": reference.file,
                "sha256": hash,
                "rationale": reference.rationale,
                "source": reference.source,
                "fasta": fasta,
            })
        })
        .collect();
    serde_json::json!({
        "bank_id": selection.motifs.bank_id,
        "bank_version": selection.motifs.version,
        "bank_hash": selection.motifs_checksum,
        "presets_hash": selection.presets_checksum,
        "preset": selection.preset_name,
        "preset_hash": effective.preset_hash,
        "enabled_entries": enabled_entries,
        "references": references,
    })
}

/// Build contaminant bank provenance for a run.
///
/// # Errors
/// Returns an error if contaminant configs cannot be loaded or resolved.
pub fn contaminant_bank_context(
    contaminant_preset: Option<&str>,
) -> Result<Option<serde_json::Value>> {
    let selection = resolve_contaminant_selection(contaminant_preset)?;
    let effective = resolve_effective_contaminants(&selection)?;
    Ok(Some(contaminant_bank_provenance_json(
        &selection, &effective,
    )))
}
