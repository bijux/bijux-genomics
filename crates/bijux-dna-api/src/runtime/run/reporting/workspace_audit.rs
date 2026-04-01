use super::{BTreeMap, BTreeSet, Context, HashSet, MetadataCommand, Path, PathBuf, Result};

/// # Errors
/// Returns an error if policy checks fail or cannot be executed.
pub fn policy_audit() -> Result<serde_json::Value> {
    let workspace = std::env::current_dir()?;
    let mut guardrails = serde_json::Map::new();
    for crate_name in ["bijux-dna-core", "bijux-dna-engine", "bijux-dna-api"] {
        let crate_root = workspace.join("crates").join(crate_name);
        let result = bijux_dna_policies::check(
            &crate_root,
            &bijux_dna_policies::GuardrailConfig::for_crate(crate_name),
        );
        let (status, error) = match result {
            Ok(()) => ("ok", None),
            Err(err) => ("fail", Some(err.to_string())),
        };
        guardrails.insert(
            crate_name.to_string(),
            serde_json::json!({
                "status": status,
                "error": error,
            }),
        );
    }
    Ok(serde_json::json!({
        "guardrails": guardrails,
    }))
}

/// # Errors
/// Returns an error if workspace dependency metadata cannot be loaded.
pub fn workspace_edges() -> Result<BTreeSet<(String, String)>> {
    let metadata = MetadataCommand::default()
        .exec()
        .context("exec cargo metadata")?;
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
                let from = id_to_name
                    .get(&id)
                    .cloned()
                    .unwrap_or_else(|| id.to_string());
                let to = id_to_name
                    .get(&dep_id)
                    .cloned()
                    .unwrap_or_else(|| dep_id.to_string());
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
