use anyhow::Result;

#[test]
fn contaminant_bank_parses() -> Result<()> {
    crate::support::with_repo_root(|| {
        let motifs_path = bijux_api::v1::api::bench::contaminant_motifs_path();
        let presets_path = bijux_api::v1::api::bench::contaminant_presets_path();
        let refs_dir = bijux_api::v1::api::bench::contaminant_references_dir();
        let motifs = bijux_api::v1::api::bench::load_contaminant_motifs(&motifs_path)?;
        let _presets =
            bijux_api::v1::api::bench::load_contaminant_presets(&presets_path, &motifs, &refs_dir)?;
        Ok(())
    })
}
