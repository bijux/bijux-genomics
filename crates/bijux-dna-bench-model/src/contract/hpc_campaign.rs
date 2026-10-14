//! Admission of declared campaign configuration source without environment or IO.
use anyhow::{anyhow, Result};

pub(crate) fn is_secret_key_name(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    if is_secret_reference_key(name) {
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
/// Returns key-only confidentiality diagnostics or refuses malformed TOML without
/// including source text or confidential values in the error.
pub fn validate_hpc_campaign_config_source(raw_toml: &str) -> Result<()> {
    let document: toml::Value = toml::from_str(raw_toml)
        .map_err(|_| anyhow!("confidential config validation failed: malformed TOML source"))?;
    let mut failures = Vec::new();
    if raw_toml.lines().any(|line| {
        let trimmed = line.trim();
        !(trimmed.starts_with('#') || trimmed.starts_with('[') && trimmed.ends_with(']'))
            && trimmed.contains("BEGIN PRIVATE KEY")
    }) {
        failures.push("configuration embeds private key material".to_string());
    }
    inspect_campaign_config_value(&document, true, false, None, None, &mut failures);
    failures.sort();
    failures.dedup();
    if failures.is_empty() {
        Ok(())
    } else {
        Err(anyhow!("confidential config validation failed:\n{}", failures.join("\n")))
    }
}

fn inspect_campaign_config_value(
    value: &toml::Value,
    root: bool,
    slurm: bool,
    secret_key: Option<&str>,
    slurm_key: Option<&str>,
    failures: &mut Vec<String>,
) {
    match value {
        toml::Value::Table(table) => {
            for (key, value) in table {
                let next_secret_key = if is_secret_reference_key(key) {
                    None
                } else if is_secret_key_name(key) {
                    Some(key.as_str())
                } else {
                    secret_key
                };
                let next_slurm_key = if slurm
                    && ["account", "project", "mail_user"]
                        .iter()
                        .any(|name| key.eq_ignore_ascii_case(name))
                {
                    Some(key.as_str())
                } else {
                    slurm_key
                };
                inspect_campaign_config_value(
                    value,
                    false,
                    root && key.eq_ignore_ascii_case("slurm"),
                    next_secret_key,
                    next_slurm_key,
                    failures,
                );
            }
        }
        toml::Value::Array(values) => {
            for value in values {
                inspect_campaign_config_value(value, false, slurm, secret_key, slurm_key, failures);
            }
        }
        value => {
            if let toml::Value::String(text) = value {
                if text.contains("BEGIN PRIVATE KEY") {
                    failures.push("configuration embeds private key material".to_string());
                }
                if text.is_empty() || looks_placeholder(text) {
                    return;
                }
            }
            if let Some(key) = slurm_key {
                failures.push(format!(
                    "field stores confidential slurm `{}` in tracked config; use env file",
                    key.to_ascii_lowercase()
                ));
            }
            if let Some(key) = secret_key {
                failures.push(format!(
                    "field stores sensitive key `{}` in tracked config",
                    key.to_ascii_lowercase()
                ));
            }
        }
    }
}

fn is_secret_reference_key(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    ["secret_provider", "secret_ref", "secret_env_key", "encryption_recipients"]
        .iter()
        .any(|allowed| lower.ends_with(allowed))
}
