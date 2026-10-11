use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{load_container_catalog, ContainerCatalogTool, CONTAINER_CATALOG_PATH};

pub const CONTAINER_REGISTRY_PROJECTION_SCHEMA_V1: &str = "bijux.container.registry_drift.v1";

const REGISTRY_PATHS: [&str; 5] = [
    "configs/ci/registry/tool_registry.toml",
    "configs/ci/registry/tool_registry_experimental.toml",
    "configs/ci/registry/tool_registry_container_experimental.toml",
    "configs/ci/registry/tool_registry_vcf.toml",
    "configs/ci/registry/tool_registry_vcf_downstream.toml",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContainerRegistryProjectionSource {
    pub path: String,
    pub sha256: String,
    pub tool_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContainerRegistryProjectionReport {
    pub schema_version: String,
    pub catalog: ContainerRegistryProjectionSource,
    pub tool_id_manifest_matches_catalog_order: bool,
    pub projected_tool_count: usize,
    pub unprojected_tool_ids: Vec<String>,
    pub registries: Vec<ContainerRegistryProjectionSource>,
    pub violation_count: usize,
    pub violations: Vec<String>,
    pub passed: bool,
}

/// Derive catalog-to-runtime-registry projection evidence from governed repository sources.
///
/// # Errors
/// Returns an error when the catalog, a registry, or the tool ID manifest cannot be read or parsed.
pub fn derive_container_registry_projection_report(
    repository_root: &Path,
) -> Result<ContainerRegistryProjectionReport> {
    let catalog_path = repository_root.join(CONTAINER_CATALOG_PATH);
    let catalog_raw = fs::read(&catalog_path)
        .with_context(|| format!("read container catalog {}", catalog_path.display()))?;
    let catalog = load_container_catalog(repository_root)?;
    let catalog_order = catalog.tools.iter().map(|tool| tool.id.clone()).collect::<Vec<_>>();
    let catalog_by_id =
        catalog.tools.iter().map(|tool| (tool.id.as_str(), tool)).collect::<BTreeMap<_, _>>();

    let mut violations = Vec::new();
    let mut registries = Vec::new();
    let mut projected_ids = BTreeSet::new();
    for relative_path in REGISTRY_PATHS {
        let path = repository_root.join(relative_path);
        let raw = fs::read_to_string(&path)
            .with_context(|| format!("read tool registry {}", path.display()))?;
        let document: toml::Value = toml::from_str(&raw)
            .with_context(|| format!("parse tool registry {}", path.display()))?;
        let rows =
            document.get("tools").and_then(toml::Value::as_array).cloned().unwrap_or_default();
        let mut registry_ids = BTreeSet::new();
        for row in &rows {
            let Some(registry) = row.as_table() else {
                violations.push(format!("{relative_path}: non-table tool row"));
                continue;
            };
            let tool_id = table_string(registry, "id");
            if !registry_ids.insert(tool_id.clone()) {
                violations.push(format!("{relative_path}: duplicate tool {tool_id:?}"));
                continue;
            }
            let Some(contract) = catalog_by_id.get(tool_id.as_str()) else {
                violations.push(format!("{relative_path}: unknown tool {tool_id:?}"));
                continue;
            };
            projected_ids.insert(tool_id);
            compare_registry_row(&mut violations, relative_path, registry, contract);
        }
        registries.push(ContainerRegistryProjectionSource {
            path: relative_path.to_string(),
            sha256: bijux_dna_infra::hash_bytes_sha256(raw.as_bytes()),
            tool_count: rows.len(),
        });
    }

    let unprojected_tool_ids = catalog_by_id
        .keys()
        .filter(|tool_id| !projected_ids.contains(**tool_id))
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    for tool_id in &unprojected_tool_ids {
        if catalog_by_id
            .get(tool_id.as_str())
            .is_some_and(|tool| tool.legacy_support_disposition() == "retained")
        {
            violations
                .push(format!("retained catalog tool {tool_id:?} has no registry projection"));
        }
    }

    let manifest_path = repository_root.join("containers/TOOL_IDS.txt");
    let manifest_raw = fs::read_to_string(&manifest_path)
        .with_context(|| format!("read tool ID manifest {}", manifest_path.display()))?;
    let manifest_order = manifest_raw
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
        .filter_map(|line| line.split_once('\t').map(|(tool_id, _)| tool_id.to_string()))
        .collect::<Vec<_>>();
    let manifest_matches = manifest_order == catalog_order;
    if !manifest_matches {
        violations
            .push("containers/TOOL_IDS.txt does not preserve exact catalog order".to_string());
    }

    Ok(ContainerRegistryProjectionReport {
        schema_version: CONTAINER_REGISTRY_PROJECTION_SCHEMA_V1.to_string(),
        catalog: ContainerRegistryProjectionSource {
            path: CONTAINER_CATALOG_PATH.to_string(),
            sha256: bijux_dna_infra::hash_bytes_sha256(&catalog_raw),
            tool_count: catalog_by_id.len(),
        },
        tool_id_manifest_matches_catalog_order: manifest_matches,
        projected_tool_count: projected_ids.len(),
        unprojected_tool_ids,
        registries,
        violation_count: violations.len(),
        passed: violations.is_empty(),
        violations,
    })
}

fn compare_registry_row(
    violations: &mut Vec<String>,
    registry_path: &str,
    registry: &toml::map::Map<String, toml::Value>,
    catalog: &ContainerCatalogTool,
) {
    let tool_id = table_string(registry, "id");
    for (registry_field, expected) in [
        ("version", catalog.version.as_str()),
        ("default_version", catalog.version.as_str()),
        ("upstream", catalog.upstream.as_str()),
        ("license", catalog.license.as_str()),
        ("expected_bin", catalog.expected_bin.as_str()),
        ("version_cmd", catalog.version_command.as_str()),
        ("smoke_version_cmd", catalog.version_command.as_str()),
        ("help_cmd", catalog.help_command.as_str()),
        ("smoke_help_cmd", catalog.help_command.as_str()),
        ("expected_version_regex", catalog.expected_version_regex.as_str()),
        ("dockerfile", catalog.dockerfile.as_str()),
        ("apptainer_def", catalog.apptainer_definition.as_str()),
    ] {
        compare_text_field(violations, registry_path, &tool_id, registry, registry_field, expected);
    }

    let mut runtimes = Vec::new();
    if catalog.docker_arm64_supported {
        runtimes.push("docker".to_string());
    }
    if catalog.apptainer_supported {
        runtimes.push("apptainer".to_string());
    }
    let actual_runtimes = table_array_strings(registry, "runtimes");
    if actual_runtimes != runtimes {
        violations.push(format!(
            "{registry_path}: {tool_id} runtimes expected {runtimes:?}, observed {actual_runtimes:?}"
        ));
    }
    let expected_container = !runtimes.is_empty();
    if table_bool(registry, "container") != expected_container {
        violations
            .push(format!("{registry_path}: {tool_id} container must be {expected_container}"));
    }
    let expected_ref = if expected_container {
        format!("bijuxdna/{tool_id}:{}", catalog.version)
    } else {
        "external@licensed-installation".to_string()
    };
    compare_text_field(
        violations,
        registry_path,
        &tool_id,
        registry,
        "container_ref",
        &expected_ref,
    );
    compare_text_field(
        violations,
        registry_path,
        &tool_id,
        registry,
        "pinned_commit",
        &expected_source_identity(catalog),
    );
    compare_text_field(
        violations,
        registry_path,
        &tool_id,
        registry,
        "pin_strategy",
        expected_pin_strategy(catalog),
    );

    if catalog.legacy_support_disposition() != "retained" {
        compare_text_field(
            violations,
            registry_path,
            &tool_id,
            registry,
            "status",
            catalog.legacy_support_disposition(),
        );
        if table_bool(registry, "container")
            || !table_string(registry, "dockerfile").is_empty()
            || !table_string(registry, "apptainer_def").is_empty()
        {
            violations.push(format!(
                "{registry_path}: {tool_id} {} tool claims a container recipe",
                catalog.legacy_support_disposition()
            ));
        }
    }
}

fn expected_source_identity(tool: &ContainerCatalogTool) -> String {
    if !tool.source_sha256.is_empty() {
        return format!("sha256:{}", tool.source_sha256);
    }
    if !tool.pinned_commit.is_empty() {
        return format!("git:{}", tool.pinned_commit);
    }
    "unresolved".to_string()
}

fn expected_pin_strategy(tool: &ContainerCatalogTool) -> &'static str {
    if !tool.source_sha256.is_empty() {
        "source_sha256"
    } else if !tool.pinned_commit.is_empty() {
        "source_commit"
    } else {
        "unresolved"
    }
}

fn compare_text_field(
    violations: &mut Vec<String>,
    registry_path: &str,
    tool_id: &str,
    registry: &toml::map::Map<String, toml::Value>,
    registry_field: &str,
    expected: &str,
) {
    let actual = table_string(registry, registry_field);
    if actual != expected {
        violations.push(format!(
            "{registry_path}: {tool_id} {registry_field} expected {expected:?}, observed {actual:?}"
        ));
    }
}

fn table_string(table: &toml::map::Map<String, toml::Value>, key: &str) -> String {
    table.get(key).and_then(toml::Value::as_str).unwrap_or_default().trim().to_string()
}

fn table_bool(table: &toml::map::Map<String, toml::Value>, key: &str) -> bool {
    table.get(key).and_then(toml::Value::as_bool).unwrap_or(false)
}

fn table_array_strings(table: &toml::map::Map<String, toml::Value>, key: &str) -> Vec<String> {
    table
        .get(key)
        .and_then(toml::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(toml::Value::as_str)
        .map(str::to_string)
        .collect()
}
