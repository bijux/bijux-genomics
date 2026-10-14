use super::Result;
use anyhow::Context;
use cargo_metadata::MetadataCommand;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::path::{Path, PathBuf};

#[cfg(test)]
mod contracts;
mod dependency_map;
mod manifest;

pub use dependency_map::{
    write_workspace_dependency_map, CrateDependencyEdge, CrateDependencyMapReport,
    CrateDependencyNode,
};
pub use manifest::{read_workspace_crate_manifests, WorkspaceCrateManifest};

/// # Errors
/// Returns an error if policy checks fail or cannot be executed.
pub fn policy_audit() -> Result<serde_json::Value> {
    Ok(serde_json::json!({
        "policy_audit": {
            "status": "delegated",
            "owner_crate": "bijux-dna-dev",
            "policy_crate": "bijux-dna-policies",
            "reason": "bijux-dna-api exposes audit metadata but does not execute policy guardrails at runtime",
            "commands": [
                "CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-policies --test boundaries --no-default-features",
                "CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-policies --test contracts --no-default-features"
            ],
        },
    }))
}

/// # Errors
/// Returns an error if workspace dependency metadata cannot be loaded.
pub fn workspace_edges() -> Result<BTreeSet<(String, String)>> {
    let metadata = MetadataCommand::default().exec().context("exec cargo metadata")?;
    Ok(edges_from_metadata(&metadata))
}

/// Load workspace dependency edges from exactly the supplied root.
///
/// Relative roots and symlink aliases are accepted, but a member directory that
/// Cargo resolves to a parent workspace is rejected. Dependency kinds and feature
/// resolution retain Cargo metadata's default behavior.
///
/// # Errors
/// Returns an error if the source root cannot be resolved, metadata cannot be loaded,
/// or Cargo resolves a different workspace root.
pub fn workspace_edges_at(source_root: &Path) -> Result<BTreeSet<(String, String)>> {
    let source_root = source_root
        .canonicalize()
        .with_context(|| format!("resolve workspace source root {}", source_root.display()))?;
    let metadata = MetadataCommand::default()
        .manifest_path(source_root.join("Cargo.toml"))
        .current_dir(&source_root)
        .exec()
        .context("exec cargo metadata")?;
    admit_metadata_source(&source_root, &metadata)?;
    Ok(edges_from_metadata(&metadata))
}

fn admit_metadata_source(source_root: &Path, metadata: &cargo_metadata::Metadata) -> Result<()> {
    let metadata_root = metadata
        .workspace_root
        .as_std_path()
        .canonicalize()
        .context("resolve cargo metadata workspace root")?;
    anyhow::ensure!(
        metadata_root == source_root,
        "cargo metadata workspace root {} does not match requested source root {}",
        metadata_root.display(),
        source_root.display()
    );
    Ok(())
}

fn edges_from_metadata(metadata: &cargo_metadata::Metadata) -> BTreeSet<(String, String)> {
    let workspace_members: HashSet<cargo_metadata::PackageId> =
        metadata.workspace_members.iter().cloned().collect();
    let mut id_to_name = BTreeMap::new();
    for pkg in &metadata.packages {
        id_to_name.insert(pkg.id.clone(), pkg.name.clone());
    }
    let mut edges = BTreeSet::new();
    if let Some(resolve) = metadata.resolve.as_ref() {
        for node in &resolve.nodes {
            let id = node.id.clone();
            if !workspace_members.contains(&id) {
                continue;
            }
            for dep in &node.deps {
                let dep_id = dep.pkg.clone();
                if !workspace_members.contains(&dep_id) {
                    continue;
                }
                let from = id_to_name.get(&id).cloned().unwrap_or_else(|| id.to_string());
                let to = id_to_name.get(&dep_id).cloned().unwrap_or_else(|| dep_id.to_string());
                edges.insert((from, to));
            }
        }
    }
    edges
}

/// # Errors
/// Returns an error if the workspace audit artifact cannot be written.
pub fn write_workspace_audit(out_dir: &Path, dot: &str) -> Result<PathBuf> {
    bijux_dna_infra::ensure_dir(out_dir)?;
    let dot_path = out_dir.join("graph.dot");
    bijux_dna_infra::write_bytes(&dot_path, dot.as_bytes())?;
    Ok(dot_path)
}
