use std::path::Path;

use anyhow::{Context, Result};

mod models;
mod resolution;
mod validation;

pub use models::{
    contaminant_motifs_path, contaminant_presets_path, contaminant_references_dir,
    ContaminantMotifBankV1, ContaminantMotifEntryV1, ContaminantPresetV1, ContaminantPresetsV1,
    ContaminantReferenceSpecV1, EffectiveContaminantSet,
};
pub use resolution::resolve_contaminant_preset;

/// Load contaminant motifs YAML and validate its contents.
///
/// # Errors
/// Returns an error if the file cannot be read, parsed, or fails validation.
pub fn load_contaminant_motifs(path: &Path) -> Result<ContaminantMotifBankV1> {
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("read contaminant motifs {}", path.display()))?;
    let bank: ContaminantMotifBankV1 =
        bijux_dna_infra::formats::parse_yaml(&contents).context("parse contaminant motifs yaml")?;
    validation::validate_contaminant_motifs(&bank)?;
    Ok(bank)
}

/// Load contaminant presets and validate references against the bank.
///
/// # Errors
/// Returns an error if the file cannot be read, parsed, or fails validation.
pub fn load_contaminant_presets(
    path: &Path,
    bank: &ContaminantMotifBankV1,
    references_dir: &Path,
) -> Result<ContaminantPresetsV1> {
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("read contaminant presets {}", path.display()))?;
    let presets: ContaminantPresetsV1 = bijux_dna_infra::formats::parse_yaml(&contents)
        .context("parse contaminant presets yaml")?;
    validation::validate_contaminant_presets(&presets, bank, references_dir)?;
    Ok(presets)
}
