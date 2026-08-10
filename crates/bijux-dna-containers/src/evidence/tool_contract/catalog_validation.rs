//! Fail-closed validation for the canonical container catalog and recipe closure.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use anyhow::Result;

use super::{
    load_container_catalog, ContainerArtifactDelivery, ContainerCatalog, ContainerProductSupport,
};

const CATALOG_SCHEMA: &str = "bijux.container.catalog.v1";
const EVIDENCE_SCHEMA: &str = "bijux.container.evidence.v1";
const PIPELINE_SCHEMA_PATH: &str = "containers/contracts/config-pipeline.schema.json";
const CREDENTIAL_SCHEMA_PATH: &str = "containers/contracts/config-credential.schema.json";
const GOVERNANCE_SCHEMA_PATH: &str = "containers/contracts/config-governance.schema.json";
const LUNARC_GOVERNANCE_PATH: &str = "containers/contracts/sites/lunarc.yml";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainerCatalogValidation {
    pub tool_count: usize,
    pub retained_count: usize,
    pub unsupported_count: usize,
    pub unresolved_source_identities: usize,
    pub errors: Vec<String>,
}

impl ContainerCatalogValidation {
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }
}

/// Validate the canonical catalog, governed contract assets, and complete recipe closure.
///
/// # Errors
/// Returns an error when the catalog or a required recipe directory cannot be read.
pub fn validate_container_catalog(repository_root: &Path) -> Result<ContainerCatalogValidation> {
    let catalog = load_container_catalog(repository_root)?;
    let mut errors = Vec::new();
    validate_header(repository_root, &catalog, &mut errors);
    validate_tools(repository_root, &catalog, &mut errors);
    validate_recipe_closure(repository_root, &catalog, &mut errors)?;

    let retained_count = count_disposition(&catalog, "retained");
    let unsupported_count = count_disposition(&catalog, "unsupported");
    let supported_count = catalog.tools.iter().filter(|tool| tool.is_product_supported()).count();
    if catalog.tools.len() != 89
        || supported_count != 89
        || retained_count != 85
        || unsupported_count != 4
    {
        errors.push(format!(
            "catalog roster must contain 89 supported tools, 85 redistributable images, and 4 external deliveries; observed {}, {}, {}, and {}",
            catalog.tools.len(), supported_count, retained_count, unsupported_count
        ));
    }
    let unresolved_source_identities = catalog
        .tools
        .iter()
        .filter(|tool| tool.legacy_support_disposition() == "retained")
        .filter(|tool| {
            (tool.source_sha256.is_empty() || is_placeholder_digest(&tool.source_sha256))
                && (tool.pinned_commit.is_empty() || is_placeholder_digest(&tool.pinned_commit))
        })
        .count();
    errors.sort();
    errors.dedup();
    Ok(ContainerCatalogValidation {
        tool_count: catalog.tools.len(),
        retained_count,
        unsupported_count,
        unresolved_source_identities,
        errors,
    })
}

fn validate_header(repository_root: &Path, catalog: &ContainerCatalog, errors: &mut Vec<String>) {
    for (valid, message) in [
        (
            catalog.schema_version == CATALOG_SCHEMA,
            format!("schema_version must be {CATALOG_SCHEMA}"),
        ),
        (!catalog.owner.trim().is_empty(), "owner must not be empty".to_string()),
        (catalog.contact == "bijan@bijux.io", "contact must be bijan@bijux.io".to_string()),
        (
            catalog.repository == "https://github.com/bijux/bijux-genomics",
            "repository must identify bijux/bijux-genomics".to_string(),
        ),
        (
            catalog.apptainer_architecture == "linux/amd64",
            "apptainer_architecture must be linux/amd64 for LUNARC".to_string(),
        ),
        (
            catalog.docker_architecture == "linux/arm64",
            "docker_architecture must be linux/arm64".to_string(),
        ),
        (
            catalog.evidence_schema == EVIDENCE_SCHEMA,
            format!("evidence_schema must be {EVIDENCE_SCHEMA}"),
        ),
        (
            catalog.recipe_license == "GPL-3.0-only",
            "recipe_license must identify the repository recipe license".to_string(),
        ),
    ] {
        if !valid {
            errors.push(message);
        }
    }

    for relative in [PIPELINE_SCHEMA_PATH, CREDENTIAL_SCHEMA_PATH, GOVERNANCE_SCHEMA_PATH] {
        let path = repository_root.join(relative);
        match fs::read(&path).map_err(anyhow::Error::from).and_then(|bytes| {
            serde_json::from_slice::<serde_json::Value>(&bytes).map_err(Into::into)
        }) {
            Ok(schema) if schema.get("$schema").and_then(serde_json::Value::as_str).is_some() => {}
            Ok(_) => errors.push(format!("{relative}: missing JSON Schema declaration")),
            Err(error) => errors.push(format!("{relative}: invalid JSON Schema: {error}")),
        }
    }
    if !repository_root.join(LUNARC_GOVERNANCE_PATH).is_file() {
        errors.push(format!("missing {LUNARC_GOVERNANCE_PATH}"));
    }

    let required_checks = strings(["help", "minimal", "negative", "real_sample_slurm", "version"]);
    if catalog.required_checks.iter().cloned().collect::<BTreeSet<_>>() != required_checks {
        errors.push(
            "required_checks must contain version, help, minimal, negative, and real_sample_slurm"
                .to_string(),
        );
    }
    let evidence_dimensions = strings([
        "benchmark",
        "build",
        "catalog",
        "freshness",
        "real_data",
        "release",
        "runtime",
        "science",
        "security",
        "source",
        "stage",
    ]);
    if catalog.evidence_dimensions.iter().cloned().collect::<BTreeSet<_>>() != evidence_dimensions {
        errors.push("evidence_dimensions must name every production dimension".to_string());
    }
}

fn validate_tools(repository_root: &Path, catalog: &ContainerCatalog, errors: &mut Vec<String>) {
    let mut seen = BTreeSet::new();
    for tool in &catalog.tools {
        if !seen.insert(tool.id.clone()) {
            errors.push(format!("{}: duplicate tool id", tool.id));
        }
        if !is_snake_component(&tool.id) {
            errors.push(format!("{}: tool id must use lowercase snake case", tool.id));
        }
        if tool.display_name.trim().is_empty() || tool.purpose.trim().is_empty() {
            errors.push(format!("{}: display name and purpose must not be empty", tool.id));
        }
        if tool.version.trim().is_empty() || matches!(tool.version.as_str(), "planned" | "unknown")
        {
            errors.push(format!("{}: version must be concrete", tool.id));
        }
        if tool.product_support == ContainerProductSupport::Supported
            && tool.artifact_delivery == ContainerArtifactDelivery::RedistributableImage
            && !tool.delivery_reason.is_empty()
        {
            errors.push(format!(
                "{}: redistributable tool must not carry a delivery reason",
                tool.id
            ));
        }
        if tool.artifact_delivery != ContainerArtifactDelivery::RedistributableImage
            && tool.delivery_reason.trim().is_empty()
        {
            errors.push(format!("{}: external delivery must carry a delivery reason", tool.id));
        }
        validate_external_artifact_support(tool, errors);
        if !tool.upstream.starts_with("https://") {
            errors.push(format!("{}: upstream must be an https URL", tool.id));
        }
        if !tool.documentation.starts_with("https://") {
            errors.push(format!("{}: documentation must be an https URL", tool.id));
        }
        if tool.license.trim().is_empty()
            || (tool.redistribution_permitted && tool.license == "NOASSERTION")
        {
            errors.push(format!("{}: retained distribution requires an explicit license", tool.id));
        }
        if !tool.source_sha256.is_empty() && !is_hex(&tool.source_sha256, 64) {
            errors.push(format!("{}: source_sha256 must be 64 hexadecimal characters", tool.id));
        }
        if !tool.pinned_commit.is_empty()
            && !is_zero_digest(&tool.pinned_commit)
            && !matches!(tool.pinned_commit.len(), 7 | 40)
        {
            errors.push(format!("{}: pinned_commit must be a Git commit identity", tool.id));
        }
        validate_sorted_unique(&tool.domains, "domains", &tool.id, errors);
        validate_sorted_unique(&tool.stage_ids, "stage_ids", &tool.id, errors);
        if tool.expected_bin.trim().is_empty()
            || tool.version_command.trim().is_empty()
            || tool.help_command.trim().is_empty()
            || tool.expected_version_regex.trim().is_empty()
        {
            errors.push(format!("{}: runtime command contract is incomplete", tool.id));
        }
        if tool.real_test_stage.trim().is_empty() || tool.real_test_input_kind.trim().is_empty() {
            errors.push(format!("{}: real-test selection is incomplete", tool.id));
        }
        validate_recipe_paths(repository_root, tool, errors);
    }

    let order = catalog.tools.iter().map(|tool| tool.id.as_str()).collect::<Vec<_>>();
    let mut sorted = order.clone();
    sorted.sort_unstable();
    if order != sorted {
        errors.push("tools must be sorted by canonical id".to_string());
    }
}

fn validate_external_artifact_support(
    tool: &super::ContainerCatalogTool,
    errors: &mut Vec<String>,
) {
    if tool.external_artifact_supported {
        if !tool.is_product_supported()
            || !tool.permits_external_artifact()
            || tool.redistribution_permitted
            || tool.apptainer_supported
            || tool.docker_arm64_supported
        {
            errors.push(format!(
                "{}: external artifact support requires supported, non-redistributable external delivery",
                tool.id
            ));
        }
        validate_sorted_unique(
            &tool.external_artifact_kinds,
            "external_artifact_kinds",
            &tool.id,
            errors,
        );
        if tool.external_artifact_kinds.is_empty()
            || tool
                .external_artifact_kinds
                .iter()
                .any(|kind| !matches!(kind.as_str(), "binary" | "apptainer_sif"))
        {
            errors.push(format!(
                "{}: external artifact kinds must select binary and/or apptainer_sif",
                tool.id
            ));
        }
    } else if !tool.external_artifact_kinds.is_empty() {
        errors.push(format!(
            "{}: external artifact kinds require external artifact support",
            tool.id
        ));
    }
    if tool.permits_external_artifact() && !tool.external_artifact_supported {
        errors.push(format!("{}: external delivery must expose a governed artifact path", tool.id));
    }
    if tool.is_redistributable_image() != tool.redistribution_permitted {
        errors
            .push(format!("{}: artifact delivery contradicts redistribution permission", tool.id));
    }
}

fn validate_recipe_paths(
    repository_root: &Path,
    tool: &super::ContainerCatalogTool,
    errors: &mut Vec<String>,
) {
    if tool.apptainer_supported {
        let expected = format!("containers/apptainer/{}.def", tool.id);
        if tool.apptainer_definition != expected || !repository_root.join(&expected).is_file() {
            errors.push(format!(
                "{}: missing or mismatched Apptainer definition {expected}",
                tool.id
            ));
        }
    } else if tool.is_redistributable_image() || !tool.apptainer_definition.is_empty() {
        errors.push(format!("{}: unsupported Apptainer contract is inconsistent", tool.id));
    }
    if tool.docker_arm64_supported {
        let expected = format!("containers/docker/Dockerfile.arm64.{}", tool.id);
        if !tool.docker_arm64_reason.is_empty()
            || tool.dockerfile != expected
            || !repository_root.join(&expected).is_file()
        {
            errors.push(format!("{}: Docker ARM64 contract is inconsistent", tool.id));
        }
    } else if !tool.dockerfile.is_empty()
        || (tool.is_redistributable_image() && tool.docker_arm64_reason.trim().is_empty())
    {
        errors.push(format!("{}: unsupported Docker ARM64 contract is inconsistent", tool.id));
    }
}

fn validate_recipe_closure(
    repository_root: &Path,
    catalog: &ContainerCatalog,
    errors: &mut Vec<String>,
) -> Result<()> {
    let catalog_apptainer = catalog
        .tools
        .iter()
        .filter(|tool| tool.apptainer_supported)
        .map(|tool| tool.id.clone())
        .collect::<BTreeSet<_>>();
    let catalog_docker = catalog
        .tools
        .iter()
        .filter(|tool| tool.docker_arm64_supported)
        .map(|tool| tool.id.clone())
        .collect::<BTreeSet<_>>();
    compare_closure(
        "Apptainer",
        &recipe_ids(&repository_root.join("containers/apptainer"), "", ".def")?,
        &catalog_apptainer,
        errors,
    );
    compare_closure(
        "Docker",
        &recipe_ids(&repository_root.join("containers/docker"), "Dockerfile.arm64.", "")?,
        &catalog_docker,
        errors,
    );
    Ok(())
}

fn recipe_ids(directory: &Path, prefix: &str, suffix: &str) -> Result<BTreeSet<String>> {
    let mut ids = BTreeSet::new();
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let Some(name) = entry.file_name().to_str().map(ToOwned::to_owned) else {
            continue;
        };
        if let Some(id) = name.strip_prefix(prefix).and_then(|value| value.strip_suffix(suffix)) {
            if !id.is_empty() {
                ids.insert(id.to_string());
            }
        }
    }
    Ok(ids)
}

fn compare_closure(
    runtime: &str,
    recipes: &BTreeSet<String>,
    catalog: &BTreeSet<String>,
    errors: &mut Vec<String>,
) {
    for missing in recipes.difference(catalog) {
        errors.push(format!("{missing}: {runtime} recipe is absent from catalog"));
    }
    for missing in catalog.difference(recipes) {
        errors.push(format!("{missing}: catalog {runtime} recipe is absent from repository"));
    }
}

fn strings<const N: usize>(values: [&str; N]) -> BTreeSet<String> {
    values.into_iter().map(ToOwned::to_owned).collect()
}

fn count_disposition(catalog: &ContainerCatalog, disposition: &str) -> usize {
    catalog.tools.iter().filter(|tool| tool.legacy_support_disposition() == disposition).count()
}

fn is_snake_component(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

fn is_zero_digest(value: &str) -> bool {
    !value.is_empty() && value.bytes().all(|byte| byte == b'0')
}

fn is_placeholder_digest(value: &str) -> bool {
    is_zero_digest(value)
        || value.bytes().next().is_some_and(|first| value.bytes().all(|byte| byte == first))
}

fn is_hex(value: &str, length: usize) -> bool {
    value.len() == length && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn validate_sorted_unique(values: &[String], label: &str, tool: &str, errors: &mut Vec<String>) {
    let mut expected = values.to_vec();
    expected.sort();
    expected.dedup();
    if values != expected {
        errors.push(format!("{tool}: {label} must be sorted and unique"));
    }
}
