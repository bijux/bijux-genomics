use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::Digest;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContaminantMotifBankV1 {
    pub schema_version: String,
    pub bank_id: String,
    pub version: String,
    pub motifs: Vec<ContaminantMotifEntryV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContaminantMotifEntryV1 {
    pub id: String,
    pub name: String,
    pub sequence: String,
    pub enabled_by_default: bool,
    pub rationale: String,
    pub source: String,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContaminantPresetsV1 {
    pub schema_version: String,
    pub presets: Vec<ContaminantPresetV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContaminantReferenceSpecV1 {
    pub id: String,
    pub file: String,
    pub rationale: String,
    pub source: String,
    #[serde(default)]
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContaminantPresetV1 {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub motif_ids: Vec<String>,
    #[serde(default)]
    pub references: Vec<ContaminantReferenceSpecV1>,
    pub rationale: String,
    #[serde(default)]
    pub notes: Vec<String>,
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectiveContaminantSet {
    pub preset: String,
    pub preset_hash: String,
    pub rationale: String,
    pub notes: Vec<String>,
    pub motifs: Vec<ContaminantMotifEntryV1>,
    pub enabled_ids: Vec<String>,
    pub references: Vec<ContaminantReferenceSpecV1>,
}

#[must_use]
pub fn contaminant_motifs_path() -> PathBuf {
    PathBuf::from("assets/contaminants/contaminant_motifs.v1.yaml")
}

#[must_use]
pub fn contaminant_presets_path() -> PathBuf {
    PathBuf::from("assets/contaminants/presets.v1.yaml")
}

#[must_use]
pub fn contaminant_references_dir() -> PathBuf {
    PathBuf::from("assets/contaminants/references")
}

/// Load contaminant motifs YAML and validate its contents.
///
/// # Errors
/// Returns an error if the file cannot be read, parsed, or fails validation.
pub fn load_contaminant_motifs(path: &Path) -> Result<ContaminantMotifBankV1> {
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("read contaminant motifs {}", path.display()))?;
    let bank: ContaminantMotifBankV1 =
        bijux_dna_infra::formats::parse_yaml(&contents).context("parse contaminant motifs yaml")?;
    validate_contaminant_motifs(&bank)?;
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
    validate_contaminant_presets(&presets, bank, references_dir)?;
    Ok(presets)
}

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

fn validate_contaminant_motifs(bank: &ContaminantMotifBankV1) -> Result<()> {
    if bank.motifs.is_empty() {
        return Err(anyhow!("contaminant motif bank contains no entries"));
    }
    if bank.bank_id.trim().is_empty() {
        return Err(anyhow!("contaminant motif bank missing bank_id"));
    }
    if bank.version.trim().is_empty() {
        return Err(anyhow!("contaminant motif bank missing version"));
    }
    let mut ids = BTreeSet::new();
    for motif in &bank.motifs {
        if !ids.insert(motif.id.clone()) {
            return Err(anyhow!("duplicate contaminant motif id {}", motif.id));
        }
        ensure_sequence_alphabet(&motif.sequence)?;
    }
    Ok(())
}

fn validate_contaminant_presets(
    presets: &ContaminantPresetsV1,
    bank: &ContaminantMotifBankV1,
    references_dir: &Path,
) -> Result<()> {
    for preset in &presets.presets {
        if preset.name.trim().is_empty() {
            return Err(anyhow!("contaminant preset missing name"));
        }
        for motif_id in &preset.motif_ids {
            if !bank.motifs.iter().any(|motif| motif.id == *motif_id) {
                return Err(anyhow!("unknown contaminant motif id {motif_id}"));
            }
        }
        for reference in &preset.references {
            if reference.id.trim().is_empty() {
                return Err(anyhow!("contaminant reference missing id"));
            }
            if reference.file.trim().is_empty() {
                return Err(anyhow!("contaminant reference missing file"));
            }
            let path = references_dir.join(&reference.file);
            if !path.exists() {
                return Err(anyhow!("missing contaminant reference {}", path.display()));
            }
        }
    }
    Ok(())
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

fn ensure_sequence_alphabet(sequence: &str) -> Result<()> {
    if sequence.trim().is_empty() {
        return Err(anyhow!("sequence cannot be empty"));
    }
    for ch in sequence.chars() {
        match ch.to_ascii_uppercase() {
            'A' | 'C' | 'G' | 'T' | 'N' => {}
            _ => return Err(anyhow!("invalid base {ch} in sequence")),
        }
    }
    Ok(())
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
    Ok(format!("{:x}", hasher.finalize()))
}
