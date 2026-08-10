use std::collections::BTreeMap;

use super::super::{anyhow, bail, ContainerCatalog, ContainerToolContract, Context, Path, Result};

pub(crate) fn load_container_catalog(
    domain_dir: &Path,
) -> Result<BTreeMap<String, ContainerToolContract>> {
    let workspace_root = domain_dir.parent().ok_or_else(|| {
        anyhow!("domain directory has no repository parent: {}", domain_dir.display())
    })?;
    let path = workspace_root.join("containers/contracts/catalog.toml");
    let raw = std::fs::read_to_string(&path)
        .with_context(|| format!("read container catalog {}", path.display()))?;
    let catalog: ContainerCatalog = toml::from_str(&raw)
        .with_context(|| format!("parse container catalog {}", path.display()))?;
    if catalog.schema_version != "bijux.container.catalog.v1" {
        bail!(
            "unsupported container catalog schema `{}` in {}",
            catalog.schema_version,
            path.display()
        );
    }

    let mut contracts = BTreeMap::new();
    for contract in catalog.tools {
        validate_contract(workspace_root, &path, &contract)?;
        let id = contract.id.clone();
        if contracts.insert(id.clone(), contract).is_some() {
            bail!("duplicate container catalog tool `{id}` in {}", path.display());
        }
    }
    Ok(contracts)
}

fn validate_contract(
    workspace_root: &Path,
    catalog_path: &Path,
    contract: &ContainerToolContract,
) -> Result<()> {
    for (field, value) in [
        ("id", contract.id.as_str()),
        ("version", contract.version.as_str()),
        ("product_support", contract.product_support.as_str()),
        ("artifact_delivery", contract.artifact_delivery.as_str()),
        ("upstream", contract.upstream.as_str()),
        ("license", contract.license.as_str()),
        ("expected_bin", contract.expected_bin.as_str()),
        ("version_command", contract.version_command.as_str()),
        ("help_command", contract.help_command.as_str()),
        ("expected_version_regex", contract.expected_version_regex.as_str()),
    ] {
        if value.trim().is_empty() {
            bail!("{}: tool {} has empty {field}", catalog_path.display(), contract.id);
        }
    }
    if !matches!(contract.product_support.as_str(), "supported" | "retired" | "excluded") {
        bail!(
            "{}: tool {} has invalid product support `{}`",
            catalog_path.display(),
            contract.id,
            contract.product_support
        );
    }
    if !matches!(
        contract.artifact_delivery.as_str(),
        "redistributable_image" | "external_artifact" | "private_site_build" | "site_module"
    ) {
        bail!(
            "{}: tool {} has invalid artifact delivery `{}`",
            catalog_path.display(),
            contract.id,
            contract.artifact_delivery
        );
    }
    if contract.external_artifact_supported
        && (contract.product_support != "supported"
            || contract.artifact_delivery == "redistributable_image"
            || contract.redistribution_permitted)
    {
        bail!(
            "{}: external artifact tool {} must remain supported with non-redistributable external delivery",
            catalog_path.display(),
            contract.id
        );
    }
    validate_runtime_recipe(
        workspace_root,
        catalog_path,
        contract,
        "Apptainer",
        contract.apptainer_supported,
        &contract.apptainer_definition,
    )?;
    validate_runtime_recipe(
        workspace_root,
        catalog_path,
        contract,
        "Docker arm64",
        contract.docker_arm64_supported,
        &contract.dockerfile,
    )?;
    Ok(())
}

fn validate_runtime_recipe(
    workspace_root: &Path,
    catalog_path: &Path,
    contract: &ContainerToolContract,
    runtime: &str,
    supported: bool,
    recipe: &str,
) -> Result<()> {
    if !supported {
        return Ok(());
    }
    if recipe.is_empty() {
        bail!(
            "{}: retained {runtime} support for {} has no recipe",
            catalog_path.display(),
            contract.id
        );
    }
    let recipe_path = workspace_root.join(recipe);
    if !recipe_path.is_file() {
        bail!(
            "{}: {runtime} recipe for {} is absent: {}",
            catalog_path.display(),
            contract.id,
            recipe_path.display()
        );
    }
    Ok(())
}
