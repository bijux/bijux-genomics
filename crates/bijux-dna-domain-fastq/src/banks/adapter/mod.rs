use std::path::Path;

use anyhow::{Context, Result};

mod models;
mod resolution;
mod validation;

pub use models::{
    adapter_bank_path, adapter_presets_path, AdapterBankV1, AdapterEntryV1, AdapterPresetV1,
    AdapterPresetsV1, EffectiveAdapterSet, ReadScope,
};
pub use resolution::{adapter_categories, adapters_by_category, resolve_adapter_preset};

const ADAPTER_TAGS: [&str; 9] =
    ["truseq", "nextera", "ssdna", "umi", "pcr", "custom", "nebnext", "capture", "partial"];

/// Load the adapter bank YAML and validate its contents.
///
/// # Errors
/// Returns an error if the file cannot be read, parsed, or fails validation.
pub fn load_adapter_bank(path: &Path) -> Result<AdapterBankV1> {
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("read adapter bank {}", path.display()))?;
    let bank: AdapterBankV1 =
        bijux_dna_infra::formats::parse_yaml(&contents).context("parse adapter bank yaml")?;
    validation::validate_adapter_bank(&bank)?;
    Ok(bank)
}

/// Load adapter presets and validate references against the bank.
///
/// # Errors
/// Returns an error if the file cannot be read, parsed, or fails validation.
pub fn load_adapter_presets(path: &Path, bank: &AdapterBankV1) -> Result<AdapterPresetsV1> {
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("read adapter presets {}", path.display()))?;
    let presets: AdapterPresetsV1 =
        bijux_dna_infra::formats::parse_yaml(&contents).context("parse adapter presets yaml")?;
    validation::validate_adapter_presets(&presets, bank)?;
    Ok(presets)
}
