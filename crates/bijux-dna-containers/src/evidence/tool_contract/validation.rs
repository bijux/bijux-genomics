use anyhow::{bail, Result};
use regex::Regex;

use super::labels::validate_tool_contract_labels;
use super::model::{
    ContainerToolContract, ToolImageIdentityEvidence, CONTAINER_TOOL_CONTRACT_SCHEMA_V1,
    TOOL_IMAGE_IDENTITY_EVIDENCE_SCHEMA_V1,
};

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
        if value != value.trim() || value.chars().any(char::is_control) {
            bail!("tool contract {label} is not safe for an Apptainer label");
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

/// Validate persisted runtime image identity against the requested immutable image and contract.
///
/// # Errors
/// Returns an error for stale schemas, image identity, embedded contracts, or labels.
pub fn validate_tool_image_identity_evidence(
    evidence: &ToolImageIdentityEvidence,
    expected_image_sha256: &str,
    expected_contract: &ContainerToolContract,
) -> Result<()> {
    if evidence.schema_version != TOOL_IMAGE_IDENTITY_EVIDENCE_SCHEMA_V1 {
        bail!("unsupported tool-image identity evidence schema");
    }
    validate_sha256(&evidence.image_sha256, "image")?;
    if evidence.image_sha256 != expected_image_sha256 {
        bail!("tool-image identity evidence differs from the requested image");
    }
    validate_tool_contract(&evidence.contract)?;
    if &evidence.contract != expected_contract {
        bail!("tool-image identity evidence differs from the requested contract");
    }
    validate_tool_contract_labels(&evidence.contract, &evidence.labels)
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
