use std::path::Path;

use anyhow::{Context, Result};

use bijux_dna_core::contract::Profile;

/// # Errors
/// Returns an error if the profile file cannot be read or parsed.
pub fn load_profile(path: &Path) -> Result<Profile> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("read profile {}", path.display()))?;
    let profile = toml::from_str::<Profile>(&raw)
        .with_context(|| format!("parse profile {}", path.display()))?;
    Ok(profile)
}
