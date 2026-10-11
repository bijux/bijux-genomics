//! Admission of declared campaign configuration source without environment or IO.
use anyhow::{anyhow, Result};

pub(crate) fn is_secret_key_name(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    let allowlist = ["secret_provider", "secret_ref", "secret_env_key", "encryption_recipients"];
    if allowlist.iter().any(|allowed| lower.ends_with(allowed)) {
        return false;
    }
    let needles = ["password", "token", "secret", "private_key", "apikey", "api_key", "access_key"];
    needles.iter().any(|needle| lower.contains(needle))
}

pub(crate) fn looks_placeholder(value: &str) -> bool {
    let lowered = value.to_ascii_lowercase();
    value.starts_with('<')
        || value.contains("${")
        || lowered.contains("example")
        || lowered.contains("placeholder")
        || lowered == "changeme"
        || lowered == "replace_me"
}

/// Reject embedded credentials in declared campaign configuration.
///
/// # Errors
/// Returns an error describing configuration fields containing confidential values.
pub fn validate_hpc_campaign_config_source(raw_toml: &str) -> Result<()> {
    let mut failures = Vec::new();
    let mut section = String::new();
    for (index, line) in raw_toml.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            section =
                trimmed.trim_start_matches('[').trim_end_matches(']').trim().to_ascii_lowercase();
            continue;
        }
        if trimmed.contains("BEGIN PRIVATE KEY") {
            failures.push(format!("line {} embeds private key material", index + 1));
            continue;
        }
        if let Some((key, value)) = trimmed.split_once('=') {
            let key = key.trim().to_ascii_lowercase();
            let value = value.trim().trim_matches('"').trim_matches('\'');
            if section == "slurm"
                && (key == "account" || key == "project" || key == "mail_user")
                && !value.is_empty()
                && !looks_placeholder(value)
            {
                failures.push(format!(
                    "line {} stores confidential slurm `{}` in tracked config; use env file",
                    index + 1,
                    key
                ));
            }
            if is_secret_key_name(&key) && !value.is_empty() && !looks_placeholder(value) {
                failures.push(format!(
                    "line {} stores sensitive key `{key}` in tracked config",
                    index + 1
                ));
            }
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(anyhow!("confidential config validation failed:\n{}", failures.join("\n")))
    }
}
