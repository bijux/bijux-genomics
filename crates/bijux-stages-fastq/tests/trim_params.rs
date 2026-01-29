use anyhow::Result;
use sha2::Digest;

fn params_hash(params: &serde_json::Value) -> Result<String> {
    let bytes = serde_json::to_vec(params)?;
    let mut hasher = sha2::Sha256::new();
    hasher.update(bytes);
    Ok(format!("{:x}", hasher.finalize()))
}

#[test]
fn disabling_adapter_changes_params_hash() -> Result<()> {
    let base = serde_json::json!({
        "adapter_preset": "default_adna",
        "enable_adapters": [],
        "disable_adapters": []
    });
    let disabled = serde_json::json!({
        "adapter_preset": "default_adna",
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
        "adapter_preset": "default_adna",
        "enable_adapters": [],
        "disable_adapters": []
    });
    let ssdna = serde_json::json!({
        "adapter_preset": "ssdna",
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
    let bank_path = bijux_stages_fastq::adapter_bank_path();
    let presets_path = bijux_stages_fastq::adapter_presets_path();
    let bank = bijux_stages_fastq::load_adapter_bank(&bank_path)?;
    let presets = bijux_stages_fastq::load_adapter_presets(&presets_path, &bank)?;
    let effective =
        bijux_stages_fastq::resolve_adapter_preset(&bank, &presets, "default_adna", &[], &[])?;
    let tmp = tempfile::TempDir::new()?;
    let tools_root = tmp.path().join("tools");
    let run_dirs = bijux_engine::api::prepare_tool_run_dirs(&tools_root, "fastp", "test-run")?;
    let run_dir = run_dirs
        .manifest_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("run dir missing for manifest"))?;
    let path = bijux_stages_fastq::artifacts::write_effective_adapters(
        run_dir, &effective, "bank", "presets",
    )?;
    let payload = std::fs::read_to_string(&path)?;
    assert!(payload.contains("truseq_universal"));
    assert!(payload.contains("truseq_indexed"));
    std::env::set_current_dir(prev_dir)?;
    Ok(())
}
