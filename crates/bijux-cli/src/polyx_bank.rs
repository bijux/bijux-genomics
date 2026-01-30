use anyhow::{anyhow, Result};
use bijux_engine::api::hash_file_sha256;
use bijux_stages_fastq::{
    load_polyx_bank, load_polyx_presets, polyx_bank_path, polyx_presets_path, resolve_polyx_preset,
    EffectivePolyxSet, PolyxBankV1, PolyxPresetsV1,
};

pub const DEFAULT_POLYX_PRESET: &str = "illumina_twocolor";

pub struct PolyxSelection {
    pub bank: PolyxBankV1,
    pub presets: PolyxPresetsV1,
    pub preset_name: String,
    pub bank_checksum: String,
    pub presets_checksum: String,
}

/// Resolve polyX selection from CLI options.
///
/// # Errors
/// Returns an error if polyX configs cannot be loaded or the preset is invalid.
pub fn resolve_polyx_selection(polyx_preset: Option<&str>) -> Result<PolyxSelection> {
    let preset_name = match polyx_preset {
        Some(name) if !name.trim().is_empty() => name.trim().to_string(),
        Some(_) => return Err(anyhow!("polyx preset name is empty")),
        None => DEFAULT_POLYX_PRESET.to_string(),
    };
    let bank_path = polyx_bank_path();
    let presets_path = polyx_presets_path();
    let bank = load_polyx_bank(&bank_path)?;
    let presets = load_polyx_presets(&presets_path, &bank)?;
    let bank_checksum = hash_file_sha256(&bank_path)?;
    let presets_checksum = hash_file_sha256(&presets_path)?;
    Ok(PolyxSelection {
        bank,
        presets,
        preset_name,
        bank_checksum,
        presets_checksum,
    })
}

/// Resolve the effective polyX set from a selection.
///
/// # Errors
/// Returns an error if the preset is invalid.
pub fn resolve_effective_polyx(selection: &PolyxSelection) -> Result<EffectivePolyxSet> {
    resolve_polyx_preset(&selection.bank, &selection.presets, &selection.preset_name)
}

#[must_use]
pub fn polyx_bank_provenance_json(
    selection: &PolyxSelection,
    effective: &EffectivePolyxSet,
) -> serde_json::Value {
    let enabled_entries: Vec<serde_json::Value> = effective
        .entries
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
    serde_json::json!({
        "bank_id": selection.bank.bank_id,
        "bank_version": selection.bank.version,
        "bank_hash": selection.bank_checksum,
        "presets_hash": selection.presets_checksum,
        "preset": selection.preset_name,
        "preset_hash": effective.preset_hash,
        "enabled_entries": enabled_entries,
    })
}
