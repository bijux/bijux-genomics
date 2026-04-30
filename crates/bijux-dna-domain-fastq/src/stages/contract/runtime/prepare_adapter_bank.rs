use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::artifacts::{PrepareAdapterBankReportV1, PREPARE_ADAPTER_BANK_REPORT_SCHEMA_VERSION};
use crate::banks::{
    adapter_bank_path, adapter_categories, adapter_presets_path, load_adapter_bank,
    load_adapter_presets, parse_adapter_preset_name, resolve_effective_adapters, AdapterSelection,
};

/// Prepare and fingerprint the governed adapter bank selection.
///
/// # Errors
/// Returns an error if the adapter bank or preset configuration is invalid.
pub fn prepare_adapter_bank(
    adapter_preset: Option<&str>,
    legacy_adapter_bank: Option<&str>,
    adapter_bank_file: Option<&Path>,
    enable_adapters: &[String],
    disable_adapters: &[String],
) -> Result<PrepareAdapterBankReportV1> {
    let selection = resolve_selection_with_repo_paths(
        adapter_preset,
        legacy_adapter_bank,
        adapter_bank_file,
    )?;
    let effective = resolve_effective_adapters(&selection, enable_adapters, disable_adapters)?;

    Ok(build_prepare_adapter_bank_report(
        &selection,
        &effective,
        enable_adapters,
        disable_adapters,
    ))
}

fn resolve_selection_with_repo_paths(
    adapter_preset: Option<&str>,
    legacy_adapter_bank: Option<&str>,
    adapter_bank_file: Option<&Path>,
) -> Result<AdapterSelection> {
    let repo_root = repository_root();
    let preset_name = parse_adapter_preset_name(adapter_preset, legacy_adapter_bank)?;
    let bank_path = adapter_bank_file
        .map(Path::to_path_buf)
        .unwrap_or_else(|| repo_root.join(adapter_bank_path()));
    let presets_path = repo_root.join(adapter_presets_path());
    let bank = load_adapter_bank(&bank_path)?;
    let presets = load_adapter_presets(&presets_path, &bank)?;
    let bank_checksum = bijux_dna_infra::hash_file_sha256(&bank_path)?;
    let presets_checksum = bijux_dna_infra::hash_file_sha256(&presets_path)?;
    Ok(AdapterSelection {
        bank,
        presets,
        preset_name,
        bank_checksum,
        presets_checksum,
    })
}

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf()
}

fn build_prepare_adapter_bank_report(
    selection: &AdapterSelection,
    effective: &crate::EffectiveAdapterSet,
    enable_adapters: &[String],
    disable_adapters: &[String],
) -> PrepareAdapterBankReportV1 {
    let mut all_categories = adapter_categories().into_iter().collect::<Vec<_>>();
    all_categories.sort();

    let enabled_categories = effective.preset_tags.iter().cloned().collect::<BTreeSet<_>>();
    let disabled_categories = all_categories
        .into_iter()
        .filter(|category| !enabled_categories.contains(category))
        .collect::<Vec<_>>();

    PrepareAdapterBankReportV1 {
        schema_version: PREPARE_ADAPTER_BANK_REPORT_SCHEMA_VERSION.to_string(),
        stage: "fastq.prepare_adapter_bank".to_string(),
        stage_id: "fastq.prepare_adapter_bank".to_string(),
        tool_id: "bijux".to_string(),
        bank_id: selection.bank.bank_id.clone(),
        bank_version: selection.bank.version.clone(),
        bank_hash: selection.bank_checksum.clone(),
        presets_hash: selection.presets_checksum.clone(),
        preset: selection.preset_name.clone(),
        preset_hash: effective.preset_hash.clone(),
        enabled_categories: enabled_categories.into_iter().collect(),
        disabled_categories,
        enable_adapters: enable_adapters.to_vec(),
        disable_adapters: disable_adapters.to_vec(),
        enabled_adapter_ids: effective.enabled_ids.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::prepare_adapter_bank;

    #[test]
    fn prepare_adapter_bank_resolves_default_preset() -> anyhow::Result<()> {
        let report = prepare_adapter_bank(None, None, None, &[], &[])?;
        assert_eq!(report.stage_id, "fastq.prepare_adapter_bank");
        assert!(!report.enabled_adapter_ids.is_empty());
        assert!(!report.bank_hash.is_empty());
        assert!(!report.presets_hash.is_empty());
        Ok(())
    }
}
