use anyhow::Result;
use bijux_dna_core::prelude::hashing::params_hash;

#[test]
fn disabling_adapter_changes_params_hash() -> Result<()> {
    let base = serde_json::json!({
        "adapter_bank": "preset:illumina-default",
        "enable_adapters": [],
        "disable_adapters": []
    });
    let disabled = serde_json::json!({
        "adapter_bank": "preset:illumina-default",
        "enable_adapters": [],
        "disable_adapters": ["truseq_universal"]
    });
    let base_hash = params_hash(&base)?;
    let disabled_hash = params_hash(&disabled)?;
    assert_ne!(base_hash, disabled_hash);
    Ok(())
}

#[test]
fn ssdna_preset_changes_params_hash() -> Result<()> {
    let base = serde_json::json!({
        "adapter_bank": "preset:illumina-default",
        "enable_adapters": [],
        "disable_adapters": []
    });
    let ssdna = serde_json::json!({
        "adapter_bank": "preset:ssdna-splint",
        "enable_adapters": [],
        "disable_adapters": []
    });
    let base_hash = params_hash(&base)?;
    let ssdna_hash = params_hash(&ssdna)?;
    assert_ne!(base_hash, ssdna_hash);
    Ok(())
}

#[test]
fn default_adapter_preset_writes_effective_adapters() -> Result<()> {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .ok_or_else(|| anyhow::anyhow!("repo root not found"))?;
    let prev_dir = std::env::current_dir()?;
    std::env::set_current_dir(repo_root)?;
    let bank_path = bijux_dna_domain_fastq::adapter_bank_path();
    let presets_path = bijux_dna_domain_fastq::adapter_presets_path();
    let bank = bijux_dna_domain_fastq::load_adapter_bank(&bank_path)?;
    let presets = bijux_dna_domain_fastq::load_adapter_presets(&presets_path, &bank)?;
    let effective = bijux_dna_domain_fastq::resolve_adapter_preset(
        &bank,
        &presets,
        "illumina-default",
        &[],
        &[],
    )?;
    let tmp = bijux_dna_infra::temp_dir("bijux")?;
    let run_dir = tmp.path().join("run");
    bijux_dna_infra::ensure_dir(&run_dir)?;
    let path = bijux_dna_stages_fastq::stage_specs::artifacts::write_effective_adapters(
        &run_dir, &effective, "bank", "presets",
    )?;
    let payload = std::fs::read_to_string(&path)?;
    assert!(payload.contains("truseq_universal"));
    assert!(payload.contains("truseq_indexed"));
    std::env::set_current_dir(prev_dir)?;
    Ok(())
}
