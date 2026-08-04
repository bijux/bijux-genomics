use anyhow::{bail, Result};
use regex::Regex;

use super::model::{ContainerToolContract, CONTAINER_TOOL_CONTRACT_SCHEMA_V1};

/// Validate every immutable field in an embedded tool contract.
///
/// # Errors
/// Returns an error for malformed, incomplete, unsupported, or ambiguous identities.
pub fn validate_tool_contract(contract: &ContainerToolContract) -> Result<()> {
    if contract.schema_version != CONTAINER_TOOL_CONTRACT_SCHEMA_V1 {
        bail!("unsupported tool-contract schema: {}", contract.schema_version);
    }
    for (label, value) in [
        ("tool_id", contract.tool_id.as_str()),
        ("version", contract.version.as_str()),
        ("recipe_path", contract.recipe_path.as_str()),
        ("architecture", contract.architecture.as_str()),
        ("expected_binary", contract.expected_binary.as_str()),
        ("version_command", contract.version_command.as_str()),
        ("expected_version_regex", contract.expected_version_regex.as_str()),
    ] {
        if value.trim().is_empty() {
            bail!("tool contract {label} is empty");
        }
    }
    if contract.support_disposition != "retained" {
        bail!("tool contract support disposition must be retained");
    }
    for (label, value) in [
        ("catalog", contract.catalog_sha256.as_str()),
        ("recipe", contract.recipe_sha256.as_str()),
        ("interface", contract.interface_sha256.as_str()),
        ("build evidence", contract.build_evidence_id.as_str()),
    ] {
        validate_sha256(value, label)?;
    }
    validate_git_commit(&contract.repository_commit, "repository")?;
    match (&contract.source_sha256, &contract.pinned_commit) {
        (Some(source), None) => validate_sha256(source, "source")?,
        (None, Some(commit)) => validate_git_commit(commit, "pinned source")?,
        (Some(_), Some(_)) => bail!("tool contract must declare only one source identity"),
        (None, None) => bail!("tool contract has no source identity"),
    }
    Regex::new(&contract.expected_version_regex)
        .map_err(|error| anyhow::anyhow!("tool contract version regex is invalid: {error}"))?;
    Ok(())
}

pub(super) fn validate_sha256(value: &str, label: &str) -> Result<()> {
    if value.len() != 64
        || !value.bytes().all(|byte| byte.is_ascii_hexdigit())
        || value.bytes().all(|byte| byte == b'0')
    {
        bail!("tool contract {label} is not a SHA-256 digest: {value}");
    }
    Ok(())
}

fn validate_git_commit(value: &str, label: &str) -> Result<()> {
    if value.len() != 40
        || !value.bytes().all(|byte| byte.is_ascii_hexdigit())
        || value.bytes().all(|byte| byte == b'0')
    {
        bail!("tool contract {label} is not a full Git object: {value}");
    }
    Ok(())
}
