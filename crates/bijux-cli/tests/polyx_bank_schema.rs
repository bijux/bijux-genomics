use anyhow::Result;

#[test]
fn polyx_bank_parses() -> Result<()> {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .ok_or_else(|| anyhow::anyhow!("repo root not found"))?;
    let prev_dir = std::env::current_dir()?;
    std::env::set_current_dir(repo_root)?;
    let bank_path = bijux_stages_fastq::polyx_bank_path();
    let presets_path = bijux_stages_fastq::polyx_presets_path();
    let bank = bijux_stages_fastq::load_polyx_bank(&bank_path)?;
    let _presets = bijux_stages_fastq::load_polyx_presets(&presets_path, &bank)?;
    std::env::set_current_dir(prev_dir)?;
    Ok(())
}
