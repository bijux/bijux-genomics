use super::admission::{admit_documents, valid_identity, VcfRegistrySnapshot};
use super::declarations::{RegistryDocument, RequiredToolsDocument};
use anyhow::{ensure, Context, Result};
use std::{collections::BTreeSet, path::Path};
/// Acquire and admit both authored VCF tool registries without publication.
///
/// # Errors
/// Returns malformed declarations, inconsistent binaries or unreadable source errors.
pub fn load_vcf_registry_snapshot(repo_root: &Path) -> Result<VcfRegistrySnapshot> {
    let mut documents = Vec::<RegistryDocument>::new();
    for relative in [
        "configs/ci/registry/tool_registry_vcf.toml",
        "configs/ci/registry/tool_registry_vcf_downstream.toml",
    ] {
        let path = repo_root.join(relative);
        let raw =
            std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
        documents.push(toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?);
    }
    admit_documents(documents)
}
pub(super) fn load_required_vcf_tool_ids(repo_root: &Path) -> Result<BTreeSet<String>> {
    let mut tools = BTreeSet::new();
    for relative in [
        "configs/ci/tools/required_tools_vcf.toml",
        "configs/ci/tools/required_tools_vcf_downstream.toml",
    ] {
        let path = repo_root.join(relative);
        let raw =
            std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
        let document: RequiredToolsDocument =
            toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
        let mut local = BTreeSet::new();
        for tool in document.required_tools {
            ensure!(
                valid_identity(&tool) && local.insert(tool.clone()),
                "required VCF tools must be nonempty, unpadded and unique within a document"
            );
            tools.insert(tool);
        }
    }
    Ok(tools)
}
