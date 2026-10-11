use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use anyhow::{bail, Context, Result};
use schemars::{json_schema, schema_for, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

pub const CONTAINER_CREDENTIAL_REFERENCE_SCHEMA_V1: &str =
    "bijux.container.credential_reference.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContainerCredentialReference {
    #[schemars(schema_with = "credential_reference_schema_version")]
    pub schema_version: String,
    pub credentials_required: bool,
    pub references: Vec<ContainerCredentialRecord>,
    pub redaction: ContainerCredentialRedaction,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContainerCredentialRecord {
    pub id: String,
    pub purpose: String,
    pub source_type: String,
    pub variable_names: Vec<String>,
    pub failure_behavior: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContainerCredentialRedaction {
    #[schemars(schema_with = "publish_values_prohibited")]
    pub publish_values: bool,
}

fn credential_reference_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": CONTAINER_CREDENTIAL_REFERENCE_SCHEMA_V1,
    })
}

fn publish_values_prohibited(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "boolean",
        "const": false,
    })
}

/// Load and validate one credential-reference declaration.
///
/// # Errors
/// Returns an error when the declaration cannot be read, parsed, or admitted.
pub fn load_container_credential_reference(path: &Path) -> Result<ContainerCredentialReference> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("read credential reference {}", path.display()))?;
    let contract: ContainerCredentialReference = serde_yaml::from_str(&raw)
        .with_context(|| format!("parse credential reference {}", path.display()))?;
    validate_container_credential_reference(&contract)
        .with_context(|| format!("validate credential reference {}", path.display()))?;
    Ok(contract)
}

/// Validate credential references without admitting secret values.
///
/// # Errors
/// Returns an error when identity, redaction, or reference invariants are violated.
pub fn validate_container_credential_reference(
    contract: &ContainerCredentialReference,
) -> Result<()> {
    if contract.schema_version != CONTAINER_CREDENTIAL_REFERENCE_SCHEMA_V1 {
        bail!("unsupported credential-reference schema: {}", contract.schema_version);
    }
    if contract.credentials_required == contract.references.is_empty() {
        bail!("credentials_required must agree with credential references");
    }
    if contract.redaction.publish_values {
        bail!("credential redaction must forbid publishing values");
    }

    let mut ids = BTreeSet::new();
    for (index, reference) in contract.references.iter().enumerate() {
        for (label, value) in [
            ("id", reference.id.as_str()),
            ("purpose", reference.purpose.as_str()),
            ("source_type", reference.source_type.as_str()),
            ("failure_behavior", reference.failure_behavior.as_str()),
        ] {
            if value.trim().is_empty() {
                bail!("credential reference {index} {label} must be non-empty");
            }
        }
        if !ids.insert(reference.id.as_str()) {
            bail!("credential reference ID is duplicated: {}", reference.id);
        }
        if reference.variable_names.iter().any(|name| name.trim().is_empty()) {
            bail!("credential reference {index} variable names must be non-empty");
        }
        let unique_names = reference.variable_names.iter().collect::<BTreeSet<_>>();
        if unique_names.len() != reference.variable_names.len() {
            bail!("credential reference {index} variable names must be unique");
        }
    }
    Ok(())
}

#[must_use]
pub fn container_credential_reference_schema() -> serde_json::Value {
    serde_json::to_value(schema_for!(ContainerCredentialReference))
        .unwrap_or_else(|_| serde_json::json!({}))
}

/// Write the credential-reference schema atomically.
///
/// # Errors
/// Returns an error when schema publication fails.
pub fn write_container_credential_reference_schema(path: &Path) -> Result<()> {
    bijux_dna_infra::atomic_write_json(path, &container_credential_reference_schema())?;
    Ok(())
}
