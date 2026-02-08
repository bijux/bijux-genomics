use anyhow::Result;
use crate::support;

#[test]
fn cli_polyx_bank_parses() -> Result<()> {
    support::with_repo_root(|| {
        let bank_path = bijux_api::v1::api::bench::polyx_bank_path();
        let presets_path = bijux_api::v1::api::bench::polyx_presets_path();
        let bank = bijux_api::v1::api::bench::load_polyx_bank(&bank_path)?;
        let _presets = bijux_api::v1::api::bench::load_polyx_presets(&presets_path, &bank)?;
        Ok(())
    })
}
