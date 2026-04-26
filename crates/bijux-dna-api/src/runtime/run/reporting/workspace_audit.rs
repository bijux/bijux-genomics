use super::Result;
use anyhow::Context;
use cargo_metadata::MetadataCommand;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::path::{Path, PathBuf};

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
    Ok(edges)
}

/// # Errors
/// Returns an error if the workspace audit artifact cannot be written.
pub fn write_workspace_audit(out_dir: &Path, dot: &str) -> Result<PathBuf> {
    bijux_dna_infra::ensure_dir(out_dir)?;
    let dot_path = out_dir.join("graph.dot");
    bijux_dna_infra::write_bytes(&dot_path, dot.as_bytes())?;
    Ok(dot_path)
}
