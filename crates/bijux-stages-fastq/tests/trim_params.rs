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
