use std::path::Path;

use anyhow::{Context, Result};

use bijux_core::contract::{Profile, RunId};

/// # Errors
/// Returns an error if the profile file cannot be read or parsed.
pub fn load_profile(path: &Path) -> Result<Profile> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("read profile {}", path.display()))?;
    let profile = toml::from_str::<Profile>(&raw)
        .with_context(|| format!("parse profile {}", path.display()))?;
    Ok(profile)
}

#[must_use]
pub fn new_run_id() -> RunId {
    RunId(format!("run-{}", uuid::Uuid::new_v4()))
}
