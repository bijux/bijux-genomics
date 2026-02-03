use std::fs;

use anyhow::Result;
use bijux_domain_fastq::banks::{
    resolve_adapter_selection, resolve_contaminant_selection, resolve_effective_adapters,
    resolve_effective_contaminants, resolve_effective_polyx, resolve_polyx_selection,
    DEFAULT_ADAPTER_PRESET, DEFAULT_CONTAMINANT_PRESET, DEFAULT_POLYX_PRESET,
};

#[test]
fn bank_preset_resolution_is_stable() -> Result<()> {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .ok_or_else(|| anyhow::anyhow!("repo root not found"))?;
    let prev_dir = std::env::current_dir()?;
    std::env::set_current_dir(repo_root)?;

    let adapter_default = resolve_adapter_selection(None, None, None)?;
    let adapter_default_effective = resolve_effective_adapters(&adapter_default, &[], &[])?;
    let adapter_override = resolve_adapter_selection(Some("nextera"), None, None)?;
    let adapter_override_effective = resolve_effective_adapters(&adapter_override, &[], &[])?;

    let polyx_default = resolve_polyx_selection(None)?;
    let polyx_default_effective = resolve_effective_polyx(&polyx_default)?;

    let contaminant_default = resolve_contaminant_selection(None)?;
    let contaminant_default_effective = resolve_effective_contaminants(&contaminant_default)?;

    let payload = serde_json::json!({
        "adapter_default": {
            "preset": adapter_default.preset_name,
            "expected_default": DEFAULT_ADAPTER_PRESET,
            "preset_hash": adapter_default_effective.preset_hash,
            "bank_hash": adapter_default.bank_checksum,
            "presets_hash": adapter_default.presets_checksum,
        },
        "adapter_override": {
            "preset": adapter_override.preset_name,
            "preset_hash": adapter_override_effective.preset_hash,
            "bank_hash": adapter_override.bank_checksum,
            "presets_hash": adapter_override.presets_checksum,
        },
        "polyx_default": {
            "preset": polyx_default.preset_name,
            "expected_default": DEFAULT_POLYX_PRESET,
            "preset_hash": polyx_default_effective.preset_hash,
            "bank_hash": polyx_default.bank_checksum,
            "presets_hash": polyx_default.presets_checksum,
        },
        "contaminant_default": {
            "preset": contaminant_default.preset_name,
            "expected_default": DEFAULT_CONTAMINANT_PRESET,
            "preset_hash": contaminant_default_effective.preset_hash,
            "bank_hash": contaminant_default.motifs_checksum,
            "presets_hash": contaminant_default.presets_checksum,
        }
    });

    let rendered = serde_json::to_string_pretty(&payload)?;
    let snapshot_path = manifest_dir
        .join("tests")
        .join("snapshots")
        .join("bank_preset_resolution.json");
    let snapshot = fs::read_to_string(&snapshot_path)?;
    assert_eq!(rendered.trim(), snapshot.trim());

    std::env::set_current_dir(prev_dir)?;
    Ok(())
}
