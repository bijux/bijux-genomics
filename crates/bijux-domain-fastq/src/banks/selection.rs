use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};

pub const DEFAULT_ADAPTER_PRESET: &str = "illumina-default";
pub const DEFAULT_CONTAMINANT_PRESET: &str = "illumina_default";
pub const DEFAULT_POLYX_PRESET: &str = "illumina_twocolor";

pub struct AdapterSelection {
    pub bank: crate::AdapterBankV1,
    pub presets: crate::AdapterPresetsV1,
    pub preset_name: String,
    pub bank_checksum: String,
    pub presets_checksum: String,
}

pub struct ContaminantSelection {
    pub motifs: crate::ContaminantMotifBankV1,
    pub presets: crate::ContaminantPresetsV1,
    pub preset_name: String,
    pub motifs_checksum: String,
    pub presets_checksum: String,
}

pub struct PolyxSelection {
    pub bank: crate::PolyxBankV1,
    pub presets: crate::PolyxPresetsV1,
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
    let bank_checksum = bijux_infra::hash_file_sha256(&bank_path)?;
    let presets_checksum = bijux_infra::hash_file_sha256(&presets_path)?;
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
        "bank_hash": selection.bank_checksum,
        "presets_hash": selection.presets_checksum,
        "preset": selection.preset_name,
        "preset_hash": effective.preset_hash,
        "enabled_categories": enabled,
        "disabled_categories": disabled,
        "enable_adapters": enable,
        "disable_adapters": disable,
        "enabled_entries": enabled_entries,
    })
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
    let motifs_checksum = bijux_infra::hash_file_sha256(&motifs_path)?;
    let presets_checksum = bijux_infra::hash_file_sha256(&presets_path)?;
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
                bijux_infra::hash_file_sha256(&path).unwrap_or_else(|_| "unknown".to_string());
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
    let bank_path = crate::polyx_bank_path();
    let presets_path = crate::polyx_presets_path();
    let bank = crate::load_polyx_bank(&bank_path)?;
    let presets = crate::load_polyx_presets(&presets_path, &bank)?;
    let bank_checksum = bijux_infra::hash_file_sha256(&bank_path)?;
    let presets_checksum = bijux_infra::hash_file_sha256(&presets_path)?;
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
pub fn resolve_effective_polyx(selection: &PolyxSelection) -> Result<crate::EffectivePolyxSet> {
    crate::resolve_polyx_preset(&selection.bank, &selection.presets, &selection.preset_name)
}

#[must_use]
pub fn polyx_bank_provenance_json(
    selection: &PolyxSelection,
    effective: &crate::EffectivePolyxSet,
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
    Ok(Some(adapter_bank_provenance_json(
        &selection, &effective, enable, disable,
    )))
}

/// Build polyX bank provenance for a run.
///
/// # Errors
/// Returns an error if polyX configs cannot be loaded or resolved.
pub fn polyx_bank_context(polyx_preset: Option<&str>) -> Result<Option<serde_json::Value>> {
    let selection = resolve_polyx_selection(polyx_preset)?;
    let effective = resolve_effective_polyx(&selection)?;
    Ok(Some(polyx_bank_provenance_json(&selection, &effective)))
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

fn tool_supports_polyx(tool_id: &str) -> bool {
    matches!(tool_id, "fastp")
}

#[must_use]
pub fn polyx_unsupported_warning(
    tool_id: &str,
    polyx_bank: Option<&serde_json::Value>,
    explicit: bool,
) -> Option<String> {
    if explicit && polyx_bank.is_some() && !tool_supports_polyx(tool_id) {
        return Some(format!(
            "warning: polyx preset requested but tool '{tool_id}' does not advertise polyX support"
        ));
    }
    None
}
