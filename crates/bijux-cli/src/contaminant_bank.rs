use anyhow::{anyhow, Result};
use bijux_engine::api::hash_file_sha256;
use bijux_stages_fastq::{
    contaminant_motifs_path, contaminant_presets_path, contaminant_references_dir,
    load_contaminant_motifs, load_contaminant_presets, resolve_contaminant_preset,
    ContaminantMotifBankV1, ContaminantPresetsV1, EffectiveContaminantSet,
};

pub const DEFAULT_CONTAMINANT_PRESET: &str = "illumina_default";

pub struct ContaminantSelection {
    pub motifs: ContaminantMotifBankV1,
    pub presets: ContaminantPresetsV1,
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
    let motifs_path = contaminant_motifs_path();
    let presets_path = contaminant_presets_path();
    let references_dir = contaminant_references_dir();
    let motifs = load_contaminant_motifs(&motifs_path)?;
    let presets = load_contaminant_presets(&presets_path, &motifs, &references_dir)?;
    let motifs_checksum = hash_file_sha256(&motifs_path)?;
    let presets_checksum = hash_file_sha256(&presets_path)?;
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
) -> Result<EffectiveContaminantSet> {
    let references_dir = contaminant_references_dir();
    resolve_contaminant_preset(
        &selection.motifs,
        &selection.presets,
        &selection.preset_name,
        &references_dir,
    )
}

#[must_use]
pub fn contaminant_bank_provenance_json(
    selection: &ContaminantSelection,
    effective: &EffectiveContaminantSet,
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
    let references_dir = contaminant_references_dir();
    let references: Vec<serde_json::Value> = effective
        .references
        .iter()
        .map(|reference| {
            let path = references_dir.join(&reference.file);
            let fasta = std::fs::read_to_string(&path).unwrap_or_default();
            let hash = hash_file_sha256(&path).unwrap_or_else(|_| "unknown".to_string());
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
