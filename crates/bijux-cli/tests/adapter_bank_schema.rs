use std::path::PathBuf;

#[test]
fn adapter_bank_parses() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .ok_or("repo root not found")?;
    let bank_path = repo_root.join("assets/adapters/bank.v1.yaml");
    let presets_path = repo_root.join("assets/adapters/presets.v1.yaml");
    let bank = bijux_domain_fastq::load_adapter_bank(&bank_path)?;
    let presets = bijux_domain_fastq::load_adapter_presets(&presets_path, &bank)?;
    assert!(
        !bank.adapters.is_empty(),
        "adapter bank should have entries"
    );
    assert!(
        !presets.presets.is_empty(),
        "adapter presets should have entries"
    );
    Ok(())
}
