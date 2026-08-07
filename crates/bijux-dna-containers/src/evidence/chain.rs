use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use anyhow::{bail, Context, Result};

use super::tool_contract::repository_commit_is_ancestor_of_head;
use super::{
    apptainer_recipe_identity_at_commit, container_build_evidence_matches_recipe,
    load_container_catalog, load_image_inventory_publication,
    load_runtime_verification_publication, repository_head_commit, validate_container_catalog,
    validate_image_inventory_publication, validate_runtime_verification_publication,
    BuildEvidenceStatus, ContainerCatalog, ContainerImageInventoryPublication,
    ContainerSupportDisposition, RuntimeEvidenceStatus, RuntimeVerificationPublication,
    CONTAINER_RUNTIME_VERIFICATION_PUBLICATION_SCHEMA_V2,
};

const CATALOG_PATH: &str = "containers/contracts/catalog.toml";
const INVENTORY_PATH: &str = "containers/reports/image-inventory.json";
const RUNTIME_PATH: &str = "containers/reports/runtime-verification.json";

#[derive(Debug, Clone)]
pub struct ContainerEvidenceChain {
    pub repository_commit: String,
    pub catalog_sha256: String,
    pub catalog: ContainerCatalog,
    pub inventory: ContainerImageInventoryPublication,
    pub runtime: RuntimeVerificationPublication,
}

/// Load and validate the canonical catalog, image inventory, and runtime evidence chain.
///
/// # Errors
/// Returns an error when any publication is malformed, the catalog closure is invalid, or an
/// identity or tool set differs across publications.
pub fn load_container_evidence_chain(repository_root: &Path) -> Result<ContainerEvidenceChain> {
    let catalog_validation = validate_container_catalog(repository_root)?;
    if !catalog_validation.is_valid() {
        bail!("container catalog is invalid: {}", catalog_validation.errors.join("; "));
    }
    let catalog_bytes = fs::read(repository_root.join(CATALOG_PATH))
        .with_context(|| format!("read canonical container catalog {CATALOG_PATH}"))?;
    let catalog_sha256 = bijux_dna_infra::hash_bytes_sha256(&catalog_bytes);
    let catalog = load_container_catalog(repository_root)?;
    let inventory = load_image_inventory_publication(&repository_root.join(INVENTORY_PATH))?;
    let runtime = load_runtime_verification_publication(&repository_root.join(RUNTIME_PATH))?;
    validate_container_evidence_chain(&catalog_sha256, &catalog, &inventory, &runtime)?;
    let repository_commit = repository_head_commit(repository_root)?;
    validate_container_recipe_evidence_chain(
        repository_root,
        &repository_commit,
        &catalog,
        &inventory,
    )?;
    Ok(ContainerEvidenceChain { repository_commit, catalog_sha256, catalog, inventory, runtime })
}

/// Validate that selected retained images were built from every current committed recipe input.
///
/// # Errors
/// Returns an error when inventory provenance is stale, a retained image is missing, or no
/// built/passed evidence from an ancestor commit matches the exact current definition and ordered
/// `%files` context.
pub fn validate_container_recipe_evidence_chain(
    repository_root: &Path,
    repository_commit: &str,
    catalog: &ContainerCatalog,
    inventory: &ContainerImageInventoryPublication,
) -> Result<()> {
    validate_image_inventory_publication(inventory)?;
    if inventory.repository_commit != repository_commit {
        bail!(
            "image inventory repository commit is stale: expected {repository_commit}, observed {}",
            inventory.repository_commit
        );
    }
    let inventory_tools =
        inventory.tools.iter().map(|tool| (tool.tool.as_str(), tool)).collect::<BTreeMap<_, _>>();
    for tool in catalog.tools.iter().filter(|tool| tool.support_disposition == "retained") {
        let inventory_tool = inventory_tools
            .get(tool.id.as_str())
            .ok_or_else(|| anyhow::anyhow!("image inventory omits retained tool {}", tool.id))?;
        let selected_sha256 = inventory_tool
            .selected_image_sha256
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("retained tool {} has no selected image", tool.id))?;
        let selected = inventory_tool
            .candidates
            .iter()
            .find(|candidate| candidate.admitted && candidate.sha256 == selected_sha256)
            .ok_or_else(|| {
                anyhow::anyhow!("retained tool {} has no admitted selected image", tool.id)
            })?;
        let recipe = apptainer_recipe_identity_at_commit(
            repository_root,
            repository_commit,
            &tool.apptainer_definition,
        )?;
        let mut recipe_matches = false;
        for evidence in &selected.build_evidence {
            if !matches!(evidence.status, BuildEvidenceStatus::Built | BuildEvidenceStatus::Passed)
                || !repository_commit_is_ancestor_of_head(
                    repository_root,
                    &evidence.repository_commit,
                )?
            {
                continue;
            }
            let built_recipe = apptainer_recipe_identity_at_commit(
                repository_root,
                &evidence.repository_commit,
                &tool.apptainer_definition,
            )?;
            recipe_matches = container_build_evidence_matches_recipe(evidence, &built_recipe)
                && evidence.definition_sha256 == recipe.definition_sha256
                && evidence.build_context.as_deref().unwrap_or_default() == recipe.build_context;
            if recipe_matches {
                break;
            }
        }
        if !recipe_matches || !selected.current_recipe {
            bail!("selected image for {} lacks current recipe build evidence", tool.id);
        }
    }
    Ok(())
}

/// Validate cross-publication identities and complete catalog tool closure.
///
/// # Errors
/// Returns an error when a publication is malformed or contradicts the canonical catalog,
/// selected image inventory, or another publication.
pub fn validate_container_evidence_chain(
    catalog_sha256: &str,
    catalog: &ContainerCatalog,
    inventory: &ContainerImageInventoryPublication,
    runtime: &RuntimeVerificationPublication,
) -> Result<()> {
    validate_container_inventory_catalog(catalog_sha256, catalog, inventory)?;
    validate_runtime_verification_publication(runtime)?;
    if runtime.schema_version != CONTAINER_RUNTIME_VERIFICATION_PUBLICATION_SCHEMA_V2 {
        bail!("runtime evidence chain requires embedded image identity evidence");
    }
    if runtime.catalog.sha256 != catalog_sha256 {
        bail!(
            "runtime publication catalog digest is stale: expected {catalog_sha256}, observed {}",
            runtime.catalog.sha256
        );
    }
    if runtime.inventory.sha256 != inventory.publication.raw_sha256 {
        bail!("runtime publication does not identify the admitted raw inventory");
    }
    if runtime.repository_commit != inventory.repository_commit {
        bail!("inventory and runtime publications identify different repository commits");
    }

    let inventory_tools =
        inventory.tools.iter().map(|tool| (tool.tool.as_str(), tool)).collect::<BTreeMap<_, _>>();
    let mut retained_ids = BTreeSet::new();
    for tool in &catalog.tools {
        let inventory_tool = inventory_tools
            .get(tool.id.as_str())
            .ok_or_else(|| anyhow::anyhow!("image inventory omits {}", tool.id))?;
        if tool.support_disposition != "retained" {
            continue;
        }
        retained_ids.insert(tool.id.as_str());
        let runtime_evidence = runtime.tools.get(&tool.id).ok_or_else(|| {
            anyhow::anyhow!("runtime publication omits retained tool {}", tool.id)
        })?;
        if runtime_evidence.version != tool.version
            || Some(runtime_evidence.image_sha256.as_str())
                != inventory_tool.selected_image_sha256.as_deref()
            || runtime_evidence.status != RuntimeEvidenceStatus::Passed
        {
            bail!("runtime publication identity or verdict differs for {}", tool.id);
        }
    }
    if runtime.tools.keys().map(String::as_str).collect::<BTreeSet<_>>() != retained_ids {
        bail!("runtime publication tool set differs from retained catalog tools");
    }
    Ok(())
}

/// Validate exact catalog identity and complete tool closure in an image inventory.
///
/// # Errors
/// Returns an error when the inventory is malformed, names a different catalog digest, omits or
/// adds tools, or changes a tool version, support disposition, or Apptainer capability.
pub fn validate_container_inventory_catalog(
    catalog_sha256: &str,
    catalog: &ContainerCatalog,
    inventory: &ContainerImageInventoryPublication,
) -> Result<()> {
    validate_image_inventory_publication(inventory)?;
    if inventory.catalog_sha256 != catalog_sha256 {
        bail!(
            "image inventory catalog digest is stale: expected {catalog_sha256}, observed {}",
            inventory.catalog_sha256
        );
    }

    let inventory_tools =
        inventory.tools.iter().map(|tool| (tool.tool.as_str(), tool)).collect::<BTreeMap<_, _>>();
    let catalog_ids = catalog.tools.iter().map(|tool| tool.id.as_str()).collect::<BTreeSet<_>>();
    let inventory_ids = inventory_tools.keys().copied().collect::<BTreeSet<_>>();
    if catalog_ids.len() != catalog.tools.len() || catalog_ids != inventory_ids {
        bail!("image inventory tool set differs from the container catalog");
    }

    for tool in &catalog.tools {
        let inventory_tool = inventory_tools
            .get(tool.id.as_str())
            .ok_or_else(|| anyhow::anyhow!("image inventory omits {}", tool.id))?;
        let expected_disposition = catalog_support_disposition(&tool.support_disposition)?;
        if inventory_tool.version != tool.version
            || inventory_tool.support_disposition != expected_disposition
            || inventory_tool.apptainer_supported != tool.apptainer_supported
        {
            bail!("image inventory identity differs from catalog for {}", tool.id);
        }
    }
    Ok(())
}

fn catalog_support_disposition(value: &str) -> Result<ContainerSupportDisposition> {
    match value {
        "retained" => Ok(ContainerSupportDisposition::Retained),
        "unsupported" => Ok(ContainerSupportDisposition::Unsupported),
        "retired" => Ok(ContainerSupportDisposition::Retired),
        "excluded" => Ok(ContainerSupportDisposition::Excluded),
        other => bail!("unsupported catalog disposition: {other}"),
    }
}
