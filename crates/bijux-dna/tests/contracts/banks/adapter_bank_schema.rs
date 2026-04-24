use crate::support;

#[test]
fn cli_adapter_bank_parses() -> Result<(), Box<dyn std::error::Error>> {
    support::with_repo_root(|| {
        let bank_path = bijux_dna_api::v1::api::bench::adapter_bank_path();
        let presets_path = bijux_dna_api::v1::api::bench::adapter_presets_path();
        let bank = bijux_dna_api::v1::api::bench::load_adapter_bank(&bank_path)?;
        let presets = bijux_dna_api::v1::api::bench::load_adapter_presets(&presets_path, &bank)?;
        assert!(!bank.adapters.is_empty(), "adapter bank should have entries");
        assert!(!presets.presets.is_empty(), "adapter presets should have entries");
        Ok(())
    })?;
    Ok(())
}
