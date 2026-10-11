//! Owner: bijux-dna-containers
//! Qualification registry version-aware wire decoding and compatibility normalization.

use super::admission::validate_qualification_run_reservations;
use super::identity::{RUN_NAMESPACE_MAXIMUM, RUN_NAMESPACE_MINIMUM, RUN_NAMESPACE_NEXT};
use super::{
    QualificationRunDefinition, QualificationRunNamespacePolicy, QualificationRunRegistry,
    QualificationRunRegistryEntry, QualificationRunRegistrySummary,
    QUALIFICATION_RUN_REGISTRY_SCHEMA_V3,
};
use crate::evidence::tool_contract::{ContainerArtifactDelivery, ContainerProductSupport};
use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::path::Path;

const QUALIFICATION_RUN_REGISTRY_SCHEMA_V1: &str = "bijux.container.qualification_run_registry.v1";

const QUALIFICATION_RUN_REGISTRY_SCHEMA_V2: &str = "bijux.container.qualification_run_registry.v2";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyQualificationRunRegistry {
    schema_version: String,
    catalog_sha256: String,
    historical_max_run_number: u64,
    tools: Vec<LegacyQualificationRunRegistryEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyQualificationRunRegistryV2 {
    schema_version: String,
    catalog_sha256: String,
    historical_max_run_number: u64,
    namespace: QualificationRunNamespacePolicy,
    summary: QualificationRunRegistrySummary,
    tools: Vec<LegacyQualificationRunRegistryEntry>,
    runs: Vec<QualificationRunDefinition>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyQualificationRunRegistryEntry {
    tool_id: String,
    support_disposition: String,
    run_id: Option<String>,
    stage_id: String,
    input_role: String,
    resource_profile: Option<String>,
    acceptance_adapter: Option<String>,
}

fn migrate_legacy_entries(
    entries: Vec<LegacyQualificationRunRegistryEntry>,
) -> Result<Vec<QualificationRunRegistryEntry>> {
    entries
        .into_iter()
        .map(|entry| {
            let (artifact_delivery, redistribution_permitted) =
                match entry.support_disposition.as_str() {
                    "retained" => (ContainerArtifactDelivery::RedistributableImage, true),
                    "unsupported" => (ContainerArtifactDelivery::ExternalArtifact, false),
                    other => bail!("unexpected legacy support disposition {other}"),
                };
            Ok(QualificationRunRegistryEntry {
                tool_id: entry.tool_id,
                product_support: ContainerProductSupport::Supported,
                artifact_delivery,
                redistribution_permitted,
                run_id: entry.run_id,
                stage_id: entry.stage_id,
                input_role: entry.input_role,
                resource_profile: entry.resource_profile,
                acceptance_adapter: entry.acceptance_adapter,
            })
        })
        .collect()
}

pub(super) fn decode_registry(
    bytes: &[u8],
    registry_path: &Path,
) -> Result<QualificationRunRegistry> {
    let value: toml::Value = toml::from_slice(bytes)
        .with_context(|| format!("parse qualification registry {}", registry_path.display()))?;
    let schema_version = value
        .get("schema_version")
        .and_then(toml::Value::as_str)
        .context("qualification registry lacks a schema version")?;
    let registry = if schema_version == QUALIFICATION_RUN_REGISTRY_SCHEMA_V1 {
        let legacy: LegacyQualificationRunRegistry =
            value.try_into().context("parse legacy qualification registry")?;
        let _ = legacy.schema_version;
        QualificationRunRegistry {
            schema_version: QUALIFICATION_RUN_REGISTRY_SCHEMA_V3.to_string(),
            catalog_sha256: legacy.catalog_sha256,
            historical_max_run_number: legacy.historical_max_run_number,
            namespace: QualificationRunNamespacePolicy {
                minimum: RUN_NAMESPACE_MINIMUM,
                maximum: RUN_NAMESPACE_MAXIMUM,
                next_run_number: RUN_NAMESPACE_NEXT,
                allocation_policy: "monotonic_only".to_string(),
                identity_policy: "existing_ids_immutable_attempts_reuse_run_id".to_string(),
            },
            summary: QualificationRunRegistrySummary {
                run_count: 0,
                campaign_count: 0,
                active_count: 0,
                allocated_count: 0,
                historical_count: 0,
                reserved_count: 0,
            },
            tools: migrate_legacy_entries(legacy.tools)?,
            runs: Vec::new(),
        }
    } else if schema_version == QUALIFICATION_RUN_REGISTRY_SCHEMA_V2 {
        let legacy: LegacyQualificationRunRegistryV2 =
            value.try_into().context("parse v2 qualification registry")?;
        let _ = legacy.schema_version;
        QualificationRunRegistry {
            schema_version: QUALIFICATION_RUN_REGISTRY_SCHEMA_V3.to_string(),
            catalog_sha256: legacy.catalog_sha256,
            historical_max_run_number: legacy.historical_max_run_number,
            namespace: legacy.namespace,
            summary: legacy.summary,
            tools: migrate_legacy_entries(legacy.tools)?,
            runs: legacy.runs,
        }
    } else {
        value.try_into().context("parse qualification registry")?
    };
    validate_qualification_run_reservations(&registry)?;
    Ok(registry)
}
