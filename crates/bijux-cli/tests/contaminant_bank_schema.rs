use anyhow::Result;

#[test]
fn contaminant_bank_parses() -> Result<()> {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .ok_or_else(|| anyhow::anyhow!("repo root not found"))?;
    let prev_dir = std::env::current_dir()?;
    std::env::set_current_dir(repo_root)?;
    let motifs_path = bijux_api::v1::bench::contaminant_motifs_path();
    let presets_path = bijux_api::v1::bench::contaminant_presets_path();
    let refs_dir = bijux_api::v1::bench::contaminant_references_dir();
    let motifs = bijux_api::v1::bench::load_contaminant_motifs(&motifs_path)?;
    let _presets =
        bijux_api::v1::bench::load_contaminant_presets(&presets_path, &motifs, &refs_dir)?;
    std::env::set_current_dir(prev_dir)?;
    Ok(())
}
