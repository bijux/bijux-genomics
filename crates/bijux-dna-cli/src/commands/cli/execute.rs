use std::str::FromStr;

use anyhow::{anyhow, Result};
use bijux_dna_api::v1::api::run::RuntimeKind;

/// # Errors
/// Returns an error if the runner override is not a valid runner kind.
pub fn parse_runner_override(env: Option<&str>) -> Result<Option<RuntimeKind>> {
    match env {
        None => Ok(None),
        Some(name) => Ok(Some(
            RuntimeKind::from_str(name).map_err(|_| anyhow!("unknown runner {name}"))?,
        )),
    }
}
