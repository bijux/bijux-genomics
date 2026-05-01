use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};

pub const DEFAULT_ADAPTER_PRESET: &str = "illumina-default";

pub struct AdapterSelection {
    pub bank: crate::AdapterBankV1,
    pub presets: crate::AdapterPresetsV1,
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
        ("custom-file".to_string(), PathBuf::from(path))
    } else {
        (
            parse_adapter_preset_name(adapter_preset, legacy_adapter_bank)?,
            crate::adapter_bank_path(),
        )
    };
    let presets_path = crate::adapter_presets_path();
    let bank = crate::load_adapter_bank(&bank_path)?;
    let presets = crate::load_adapter_presets(&presets_path, &bank)?;
    let bank_checksum = bijux_dna_infra::hash_file_sha256(&bank_path)?;
    let presets_checksum = bijux_dna_infra::hash_file_sha256(&presets_path)?;
    Ok(AdapterSelection { bank, presets, preset_name, bank_checksum, presets_checksum })
}

/// Resolve the effective adapter set from a selection and overrides.
///
/// # Errors
/// Returns an error if the preset or overrides are invalid.
pub fn resolve_effective_adapters(
    selection: &AdapterSelection,
    enable: &[String],
    disable: &[String],
) -> Result<crate::EffectiveAdapterSet> {
    crate::resolve_adapter_preset(
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
    effective: &crate::EffectiveAdapterSet,
    enable: &[String],
    disable: &[String],
) -> serde_json::Value {
    let mut categories: Vec<String> = crate::adapter_categories().into_iter().collect();
    categories.sort();
    let mut enabled = effective.preset_tags.clone();
    enabled.sort();
    let disabled: Vec<String> = categories
        .into_iter()
        .filter(|tag| !enabled.iter().any(|enabled_tag| enabled_tag == tag))
        .collect();
    let enabled_entries: Vec<serde_json::Value> = effective
        .adapters
        .iter()
        .map(|adapter| {
            serde_json::json!({
                "id": adapter.id,
                "sequence": adapter.sequence,
                "rationale": adapter.rationale,
                "source": adapter.source,
            })
        })
        .collect();
    serde_json::json!({
        "bank_id": selection.bank.bank_id,
        "bank_version": selection.bank.version,
        "bank_license": selection.bank.license,
        "bank_source_document": selection.bank.source_document,
        "bank_hash": selection.bank_checksum,
        "bank_source_checksum_sha256": selection.bank.source_checksum_sha256,
        "presets_hash": selection.presets_checksum,
        "preset": selection.preset_name,
        "preset_hash": effective.preset_hash,
        "preset_applicable_assays": effective.applicable_assays,
        "enabled_categories": enabled,
        "disabled_categories": disabled,
        "selection_logic": effective.selection_logic,
        "enable_adapters": enable,
        "disable_adapters": disable,
        "enabled_entries": enabled_entries,
    })
}

/// Build adapter bank provenance for a run.
///
/// # Errors
/// Returns an error if adapter configs cannot be loaded or resolved.
pub fn adapter_bank_context(
    adapter_bank_preset: Option<&str>,
    legacy_adapter_bank: Option<&str>,
    adapter_bank_file: Option<&Path>,
    enable: &[String],
    disable: &[String],
) -> Result<Option<serde_json::Value>> {
    let selection =
        resolve_adapter_selection(adapter_bank_preset, legacy_adapter_bank, adapter_bank_file)?;
    let effective = resolve_effective_adapters(&selection, enable, disable)?;
    Ok(Some(adapter_bank_provenance_json(&selection, &effective, enable, disable)))
}
