use std::fmt::Write as _;

use anyhow::{bail, Context, Result};
use serde::Serialize;
use sha2::{Digest, Sha256};

use super::catalog::{load_retained_tool, CatalogToolSource};
use super::labels::tool_contract_labels;
use super::model::{
    ContainerToolContract, PreparedToolContractBuild, ToolContractBuildRequest,
    CONTAINER_TOOL_CONTRACT_PATH, CONTAINER_TOOL_CONTRACT_SCHEMA_V3,
};
use super::validation::validate_tool_contract;

#[derive(Serialize)]
struct InterfaceIdentity<'a> {
    expected_binary: &'a str,
    version_command: &'a str,
    expected_version_regex: &'a str,
}

#[derive(Serialize)]
struct CatalogEntryIdentity<'a> {
    schema_version: &'static str,
    architecture: &'a str,
    tool: &'a super::catalog::ContainerCatalogTool,
}

/// Generate one embedded contract and a definition that installs it.
///
/// # Errors
/// Returns an error when catalog, recipe, identity, or output contracts are invalid.
pub fn prepare_apptainer_tool_contract(
    request: &ToolContractBuildRequest,
) -> Result<PreparedToolContractBuild> {
    if !request.output_root.starts_with(&request.repository_root) {
        bail!("tool-contract output root must be inside the repository");
    }
    prepare_tool_contract_build(request, true)
}

pub(crate) fn prepare_apptainer_tool_contract_for_publication(
    request: &ToolContractBuildRequest,
) -> Result<PreparedToolContractBuild> {
    if !request.output_root.is_absolute()
        || request.output_root.starts_with(&request.repository_root)
    {
        bail!("publication tool-contract output root must be absolute and outside the repository");
    }
    prepare_tool_contract_build(request, false)
}

fn prepare_tool_contract_build(
    request: &ToolContractBuildRequest,
    repository_relative_output: bool,
) -> Result<PreparedToolContractBuild> {
    let source =
        load_retained_tool(&request.repository_root, &request.tool_id, &request.architecture)?;
    let contract = derive_contract(request, &source)?;
    persist_contract_build(request, source, contract, repository_relative_output)
}

/// Derive the immutable contract identity for one retained Apptainer tool without writing files.
///
/// # Errors
/// Returns an error when the catalog, recipe, source identity, or requested architecture is
/// invalid.
pub fn derive_apptainer_tool_contract(
    repository_root: &std::path::Path,
    tool_id: &str,
    repository_commit: &str,
    architecture: &str,
) -> Result<ContainerToolContract> {
    let source = load_retained_tool(repository_root, tool_id, architecture)?;
    derive_contract(
        &ToolContractBuildRequest {
            repository_root: repository_root.to_path_buf(),
            tool_id: tool_id.to_string(),
            repository_commit: repository_commit.to_string(),
            architecture: architecture.to_string(),
            output_root: repository_root.to_path_buf(),
        },
        &source,
    )
}

fn derive_contract(
    request: &ToolContractBuildRequest,
    source: &CatalogToolSource,
) -> Result<ContainerToolContract> {
    let recipe_path = source
        .recipe_path
        .strip_prefix(&request.repository_root)
        .context("Apptainer definition is outside the repository")?
        .to_string_lossy()
        .replace('\\', "/");
    let catalog_sha256 = sha256_hex(&source.catalog_bytes);
    let catalog_entry_sha256 = sha256_json(&CatalogEntryIdentity {
        schema_version: super::catalog::CONTAINER_CATALOG_SCHEMA_V1,
        architecture: &request.architecture,
        tool: &source.tool,
    })?;
    let recipe_sha256 = sha256_hex(&source.recipe_bytes);
    let interface_sha256 = sha256_json(&InterfaceIdentity {
        expected_binary: &source.tool.expected_bin,
        version_command: &source.tool.version_command,
        expected_version_regex: &source.tool.expected_version_regex,
    })?;
    let build_evidence_id = sha256_json(&serde_json::json!({
        "architecture": request.architecture,
        "catalog_entry_sha256": catalog_entry_sha256,
        "interface_sha256": interface_sha256,
        "recipe_sha256": recipe_sha256,
        "tool_id": request.tool_id,
    }))?;
    let (source_sha256, pinned_commit) = if valid_sha256(&source.tool.source_sha256) {
        (Some(source.tool.source_sha256.clone()), None)
    } else if valid_git_commit(&source.tool.pinned_commit) {
        (None, Some(source.tool.pinned_commit.clone()))
    } else {
        bail!("retained tool {} has no valid source identity", request.tool_id);
    };
    let contract = ContainerToolContract {
        schema_version: CONTAINER_TOOL_CONTRACT_SCHEMA_V3.to_string(),
        tool_id: source.tool.id.clone(),
        version: source.tool.version.clone(),
        product_support: source.tool.product_support,
        artifact_delivery: source.tool.artifact_delivery,
        redistribution_permitted: source.tool.redistribution_permitted,
        catalog_entry_sha256,
        catalog_sha256,
        recipe_path,
        recipe_sha256,
        source_sha256,
        pinned_commit,
        repository_commit: request.repository_commit.clone(),
        architecture: request.architecture.clone(),
        build_evidence_id,
        expected_binary: source.tool.expected_bin.clone(),
        version_command: source.tool.version_command.clone(),
        expected_version_regex: source.tool.expected_version_regex.clone(),
        interface_sha256,
    };
    validate_tool_contract(&contract)?;
    Ok(contract)
}

fn persist_contract_build(
    request: &ToolContractBuildRequest,
    source: CatalogToolSource,
    contract: ContainerToolContract,
    repository_relative_output: bool,
) -> Result<PreparedToolContractBuild> {
    let tool_root = request.output_root.join(&contract.tool_id);
    bijux_dna_infra::ensure_dir(&tool_root)
        .with_context(|| format!("create tool-contract root {}", tool_root.display()))?;
    let contract_path = tool_root.join("tool-contract.json");
    bijux_dna_infra::atomic_write_json(&contract_path, &contract)
        .with_context(|| format!("write tool contract {}", contract_path.display()))?;
    let contract_source = if repository_relative_output {
        contract_path
            .strip_prefix(&request.repository_root)
            .context("tool-contract output root must be inside the repository")?
            .to_string_lossy()
            .replace('\\', "/")
    } else {
        contract_path.to_string_lossy().replace('\\', "/")
    };
    let recipe =
        String::from_utf8(source.recipe_bytes).context("Apptainer definition is not UTF-8")?;
    let rendered = render_definition(&recipe, &contract_source, &contract)?;
    let rendered_definition_path = tool_root.join(format!("{}.def", contract.tool_id));
    bijux_dna_infra::atomic_write_bytes(&rendered_definition_path, rendered.as_bytes())
        .with_context(|| {
            format!("write rendered definition {}", rendered_definition_path.display())
        })?;

    Ok(PreparedToolContractBuild { contract, contract_path, rendered_definition_path })
}

fn render_definition(
    recipe: &str,
    contract_source: &str,
    contract: &ContainerToolContract,
) -> Result<String> {
    if recipe.contains(CONTAINER_TOOL_CONTRACT_PATH) || recipe.contains("io.bijux.tool-contract.") {
        bail!("canonical recipe already contains generated tool-contract material");
    }
    let mut labels = String::new();
    for (key, value) in tool_contract_labels(contract) {
        writeln!(labels, "    {key} {value}")?;
    }
    let with_labels = insert_section_line(recipe, "labels", &labels)?;
    let file_line = format!("    {contract_source} {CONTAINER_TOOL_CONTRACT_PATH}\n");
    insert_section_line(&with_labels, "files", &file_line)
}

fn insert_section_line(recipe: &str, section: &str, addition: &str) -> Result<String> {
    let marker = format!("%{section}");
    let Some(start) = recipe.find(&marker) else {
        if section != "files" {
            bail!("Apptainer definition has no %{section} section");
        }
        let insertion = ["\n%environment", "\n%post", "\n%runscript", "\n%test"]
            .iter()
            .filter_map(|next| recipe.find(next))
            .min()
            .unwrap_or(recipe.len());
        let mut rendered = String::with_capacity(recipe.len() + marker.len() + addition.len() + 2);
        rendered.push_str(&recipe[..insertion]);
        if !rendered.ends_with('\n') {
            rendered.push('\n');
        }
        rendered.push_str(&marker);
        rendered.push('\n');
        rendered.push_str(addition);
        rendered.push_str(&recipe[insertion..]);
        return Ok(rendered);
    };
    let body_start = recipe[start..]
        .find('\n')
        .map(|offset| start + offset + 1)
        .ok_or_else(|| anyhow::anyhow!("Apptainer %{section} section has no body"))?;
    let end =
        recipe[body_start..].find("\n%").map_or(recipe.len(), |offset| body_start + offset + 1);
    let mut rendered = String::with_capacity(recipe.len() + addition.len());
    rendered.push_str(&recipe[..end]);
    if !rendered.ends_with('\n') {
        rendered.push('\n');
    }
    rendered.push_str(addition);
    rendered.push_str(&recipe[end..]);
    Ok(rendered)
}

fn sha256_json<T: Serialize>(value: &T) -> Result<String> {
    Ok(sha256_hex(&serde_json::to_vec(value)?))
}

fn sha256_hex(bytes: &[u8]) -> String {
    hex_digest(&Sha256::digest(bytes))
}

fn hex_digest(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(output, "{byte:02x}");
    }
    output
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value.bytes().all(|byte| byte.is_ascii_hexdigit())
        && !value.bytes().all(|byte| byte == b'0')
}

fn valid_git_commit(value: &str) -> bool {
    value.len() == 40
        && value.bytes().all(|byte| byte.is_ascii_hexdigit())
        && !value.bytes().all(|byte| byte == b'0')
}
