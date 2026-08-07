use std::path::Path;

use anyhow::Result;

const CONTAINER_GOVERNANCE_SCHEMA_JSON: &str = r#"{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://bijux.github.io/bijux-genomics/schemas/container-governance-v1.json",
  "title": "Bijux Genomics site-governance contract",
  "type": "object",
  "required": ["schema_version", "site", "paths", "runtime", "slurm", "preflight"],
  "properties": {
    "schema_version": {"const": "bijux.container.governance.v1"},
    "site": {
      "type": "object",
      "required": ["id", "hostname_regex"],
      "properties": {
        "id": {"type": "string", "minLength": 1},
        "hostname_regex": {"type": "string", "minLength": 1}
      },
      "additionalProperties": false
    },
    "paths": {
      "type": "object",
      "required": ["study_root", "finalization_root", "active_repository", "frozen_repositories"],
      "properties": {
        "study_root": {"type": "string", "pattern": "^/"},
        "containers_root": {"type": "string", "pattern": "^/"},
        "data_root": {"type": "string", "pattern": "^/"},
        "results_root": {"type": "string", "pattern": "^/"},
        "finalization_root": {"type": "string", "pattern": "^/"},
        "active_repository": {"type": "string", "pattern": "^/"},
        "frozen_repositories": {"type": "string", "pattern": "^/"}
      },
      "additionalProperties": false
    },
    "runtime": {
      "type": "object",
      "required": ["apptainer_architecture"],
      "properties": {
        "apptainer_architecture": {"enum": ["linux/amd64", "linux/arm64"]}
      },
      "additionalProperties": false
    },
    "slurm": {
      "type": "object",
      "required": ["account", "partitions", "quota_command"],
      "oneOf": [
        {"required": ["scratch_environment"]},
        {"required": ["scratch_root"]}
      ],
      "properties": {
        "account": {"type": "string", "minLength": 1},
        "scratch_environment": {
          "type": "string",
          "pattern": "^[A-Z][A-Z0-9_]*$"
        },
        "scratch_root": {"type": "string", "pattern": "^/"},
        "partitions": {
          "type": "array",
          "minItems": 1,
          "uniqueItems": true,
          "items": {"type": "string", "minLength": 1}
        },
        "quota_command": {
          "type": "array",
          "minItems": 1,
          "items": {"type": "string", "minLength": 1}
        }
      },
      "additionalProperties": false
    },
    "preflight": {
      "type": "object",
      "required": ["minimum_available_bytes", "minimum_available_inodes", "maximum_clock_skew_seconds", "required_commands", "required_contracts"],
      "properties": {
        "minimum_available_bytes": {"type": "integer", "minimum": 1},
        "minimum_available_inodes": {"type": "integer", "minimum": 1},
        "maximum_clock_skew_seconds": {"type": "integer", "minimum": 0},
        "required_commands": {
          "type": "array",
          "minItems": 1,
          "uniqueItems": true,
          "items": {"type": "string", "minLength": 1}
        },
        "required_contracts": {
          "type": "array",
          "minItems": 1,
          "uniqueItems": true,
          "items": {"type": "string", "minLength": 1}
        }
      },
      "additionalProperties": false
    }
  },
  "additionalProperties": false
}"#;

#[must_use]
pub fn container_governance_schema() -> serde_json::Value {
    serde_json::from_str(CONTAINER_GOVERNANCE_SCHEMA_JSON)
        .unwrap_or_else(|_| serde_json::json!({}))
}

/// Write the container-governance schema atomically.
///
/// # Errors
/// Returns an error when schema publication fails.
pub fn write_container_governance_schema(path: &Path) -> Result<()> {
    bijux_dna_infra::atomic_write_json(path, &container_governance_schema())?;
    Ok(())
}
