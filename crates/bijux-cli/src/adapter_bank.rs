use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use bijux_engine::api::hash_file_sha256;
use bijux_stages_fastq::adapter_categories;
use bijux_stages_fastq::{
    adapter_bank_path, adapter_presets_path, load_adapter_bank, load_adapter_presets,
    resolve_adapter_preset, AdapterBankV1, AdapterPresetsV1, EffectiveAdapterSet,
};

pub const DEFAULT_ADAPTER_PRESET: &str = "ancientdna-illumina";

pub struct AdapterSelection {
    pub bank: AdapterBankV1,
    pub presets: AdapterPresetsV1,
    pub preset_name: String,
    pub bank_checksum: String,
    pub presets_checksum: String,
}

/// Parse the adapter preset name from CLI input.
///
/// # Errors
/// Returns an error if the preset is empty or not in `preset:<name>` form.
pub fn parse_adapter_preset_name(
    adapter_preset: Option<&str>,
    legacy_adapter_bank: Option<&str>,
) -> Result<String> {
    if let Some(preset) = adapter_preset {
        if preset.trim().is_empty() {
            return Err(anyhow!("adapter preset name is empty"));
        }
        return Ok(preset.trim().to_string());
    }
    match legacy_adapter_bank {
        None => Ok(DEFAULT_ADAPTER_PRESET.to_string()),
        Some(raw) => {
            if let Some(name) = raw.strip_prefix("preset:") {
                if name.trim().is_empty() {
                    Err(anyhow!("adapter preset name is empty"))
                } else {
                    Ok(name.trim().to_string())
                }
            } else {
                Err(anyhow!(
                    "--adapter-bank expects preset:<name>; use --adapter-bank-file for files"
                ))
            }
        }
    }
}

/// Resolve adapter selection from CLI options.
///
/// # Errors
/// Returns an error if adapter configs cannot be loaded or the preset is invalid.
pub fn resolve_adapter_selection(
    adapter_preset: Option<&str>,
    legacy_adapter_bank: Option<&str>,
    adapter_bank_file: Option<&Path>,
) -> Result<AdapterSelection> {
    let (preset_name, bank_path) = if let Some(path) = adapter_bank_file {
        ("custom".to_string(), PathBuf::from(path))
    } else {
        (
            parse_adapter_preset_name(adapter_preset, legacy_adapter_bank)?,
            adapter_bank_path(),
        )
    };
    let presets_path = adapter_presets_path();
    let bank = load_adapter_bank(&bank_path)?;
    let presets = load_adapter_presets(&presets_path, &bank)?;
    let bank_checksum = hash_file_sha256(&bank_path)?;
    let presets_checksum = hash_file_sha256(&presets_path)?;
    Ok(AdapterSelection {
        bank,
        presets,
        preset_name,
        bank_checksum,
        presets_checksum,
    })
}

/// Resolve the effective adapter set from a selection and overrides.
///
/// # Errors
/// Returns an error if the preset or overrides are invalid.
pub fn resolve_effective_adapters(
    selection: &AdapterSelection,
    enable: &[String],
    disable: &[String],
) -> Result<EffectiveAdapterSet> {
    resolve_adapter_preset(
        &selection.bank,
        &selection.presets,
        &selection.preset_name,
        enable,
        disable,
    )
}

#[must_use]
pub fn adapter_bank_provenance_json(
    selection: &AdapterSelection,
    effective: &EffectiveAdapterSet,
    enable: &[String],
    disable: &[String],
) -> serde_json::Value {
    let mut categories: Vec<String> = adapter_categories().into_iter().collect();
    categories.sort();
    let mut enabled = effective.preset_tags.clone();
    enabled.sort();
    let disabled: Vec<String> = categories
        .into_iter()
        .filter(|tag| !enabled.iter().any(|enabled_tag| enabled_tag == tag))
        .collect();
    serde_json::json!({
        "bank_id": selection.bank.bank_id,
        "bank_version": selection.bank.version,
        "bank_hash": selection.bank_checksum,
        "presets_hash": selection.presets_checksum,
        "preset": selection.preset_name,
        "preset_hash": effective.preset_hash,
        "enabled_categories": enabled,
        "disabled_categories": disabled,
        "enable_adapters": enable,
        "disable_adapters": disable,
    })
}
