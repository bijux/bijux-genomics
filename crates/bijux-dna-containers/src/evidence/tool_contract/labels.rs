use std::collections::BTreeMap;

use anyhow::{bail, Result};

use super::model::ContainerToolContract;

pub type ToolContractLabels = BTreeMap<String, String>;

#[must_use]
pub fn tool_contract_labels(contract: &ContainerToolContract) -> ToolContractLabels {
    let mut labels = BTreeMap::from([
        ("io.bijux.tool-contract.schema".to_string(), contract.schema_version.clone()),
        ("io.bijux.tool-contract.tool-id".to_string(), contract.tool_id.clone()),
        ("io.bijux.tool-contract.version".to_string(), contract.version.clone()),
        (
            "io.bijux.tool-contract.product-support".to_string(),
            contract.product_support.as_str().to_string(),
        ),
        (
            "io.bijux.tool-contract.artifact-delivery".to_string(),
            contract.artifact_delivery.as_str().to_string(),
        ),
        (
            "io.bijux.tool-contract.redistribution-permitted".to_string(),
            contract.redistribution_permitted.to_string(),
        ),
        (
            "io.bijux.tool-contract.catalog-entry-sha256".to_string(),
            contract.catalog_entry_sha256.clone(),
        ),
        ("io.bijux.tool-contract.catalog-sha256".to_string(), contract.catalog_sha256.clone()),
        ("io.bijux.tool-contract.recipe-path".to_string(), contract.recipe_path.clone()),
        ("io.bijux.tool-contract.recipe-sha256".to_string(), contract.recipe_sha256.clone()),
        (
            "io.bijux.tool-contract.repository-commit".to_string(),
            contract.repository_commit.clone(),
        ),
        ("io.bijux.tool-contract.architecture".to_string(), contract.architecture.clone()),
        (
            "io.bijux.tool-contract.build-evidence-id".to_string(),
            contract.build_evidence_id.clone(),
        ),
        ("io.bijux.tool-contract.expected-binary".to_string(), contract.expected_binary.clone()),
        ("io.bijux.tool-contract.version-command".to_string(), contract.version_command.clone()),
        (
            "io.bijux.tool-contract.expected-version-regex".to_string(),
            contract.expected_version_regex.clone(),
        ),
        ("io.bijux.tool-contract.interface-sha256".to_string(), contract.interface_sha256.clone()),
    ]);
    if let Some(value) = &contract.source_sha256 {
        labels.insert("io.bijux.tool-contract.source-sha256".to_string(), value.clone());
    }
    if let Some(value) = &contract.pinned_commit {
        labels.insert("io.bijux.tool-contract.pinned-commit".to_string(), value.clone());
    }
    labels
}

/// Require every governed label to equal the embedded JSON contract.
///
/// # Errors
/// Returns an error for missing, stale, unexpected source, or mismatched labels.
pub fn validate_tool_contract_labels(
    contract: &ContainerToolContract,
    actual: &ToolContractLabels,
) -> Result<()> {
    let expected = tool_contract_labels(contract);
    for (key, value) in &expected {
        match actual.get(key) {
            Some(actual_value) if actual_value == value => {}
            Some(actual_value) => bail!(
                "tool-contract label {key} mismatches embedded contract: expected {value}, observed {actual_value}"
            ),
            None => bail!("tool-contract label {key} is missing"),
        }
    }
    for key in actual.keys().filter(|key| key.starts_with("io.bijux.tool-contract.")) {
        if !expected.contains_key(key) {
            bail!("unexpected tool-contract label {key}");
        }
    }
    Ok(())
}
