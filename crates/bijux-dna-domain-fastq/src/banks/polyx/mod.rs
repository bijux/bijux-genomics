use std::path::Path;

use anyhow::{Context, Result};

mod models;
mod resolution;
mod validation;

pub use models::{
    polyx_bank_path, polyx_presets_path, EffectivePolyxSet, PolyxBankV1, PolyxEntryV1,
    PolyxPresetV1, PolyxPresetsV1,
};
pub use resolution::resolve_polyx_preset;

/// Load the polyX bank YAML and validate its contents.
///
/// # Errors
/// Returns an error if the file cannot be read, parsed, or fails validation.
pub fn load_polyx_bank(path: &Path) -> Result<PolyxBankV1> {
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("read polyx bank {}", path.display()))?;
    let bank: PolyxBankV1 =
        bijux_dna_infra::formats::parse_yaml(&contents).context("parse polyx bank yaml")?;
    validation::validate_polyx_bank(&bank)?;
    Ok(bank)
}

/// Load polyX presets and validate references against the bank.
///
/// # Errors
/// Returns an error if the file cannot be read, parsed, or fails validation.
pub fn load_polyx_presets(path: &Path, bank: &PolyxBankV1) -> Result<PolyxPresetsV1> {
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("read polyx presets {}", path.display()))?;
    let presets: PolyxPresetsV1 =
        bijux_dna_infra::formats::parse_yaml(&contents).context("parse polyx presets yaml")?;
    validation::validate_polyx_presets(&presets, bank)?;
    Ok(presets)
}
