use serde_json::Value;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::process::Command;

/// # Errors
/// Returns an error if cargo metadata cannot be parsed or the DOT file cannot be written.
pub fn workspace_audit(out_dir: &Path) -> Result<()> {
    std::fs::create_dir_all(out_dir).context("create audit output dir")?;
    let output = Command::new("cargo")
        .args(["metadata", "--format-version", "1"])
        .output()
        .context("run cargo metadata")?;
    if !output.status.success() {
        return Err(anyhow!(
            "cargo metadata failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let metadata: Value = serde_json::from_slice(&output.stdout)
        .context("parse cargo metadata JSON")?;
    let workspace_members: HashSet<String> = metadata
        .get("workspace_members")
        .and_then(Value::as_array)
        .unwrap_or(&Vec::new())
        .iter()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect();
    let mut id_to_name = HashMap::new();
    if let Some(packages) = metadata.get("packages").and_then(Value::as_array) {
        for pkg in packages {
            if let (Some(id), Some(name)) = (pkg.get("id"), pkg.get("name")) {
                if let (Some(id), Some(name)) = (id.as_str(), name.as_str()) {
                    id_to_name.insert(id.to_string(), name.to_string());
                }
            }
        }
    }
    let mut edges = BTreeSet::new();
    if let Some(resolve) = metadata.get("resolve") {
        if let Some(nodes) = resolve.get("nodes").and_then(Value::as_array) {
            for node in nodes {
                let Some(id) = node.get("id").and_then(Value::as_str) else {
                    continue;
                };
                if !workspace_members.contains(id) {
                    continue;
                }
                let Some(deps) = node.get("deps").and_then(Value::as_array) else {
                    continue;
                };
                for dep in deps {
                    let Some(dep_id) = dep.get("pkg").and_then(Value::as_str) else {
                        continue;
                    };
                    if !workspace_members.contains(dep_id) {
                        continue;
                    }
                    let from = id_to_name.get(id).cloned().unwrap_or_else(|| id.to_string());
                    let to = id_to_name
                        .get(dep_id)
                        .cloned()
                        .unwrap_or_else(|| dep_id.to_string());
                    edges.insert((from, to));
                }
            }
        }
    }

    println!("allowed edges (workspace): {}", edges.len());
    for (from, to) in &edges {
        println!("  {from} -> {to}");
    }
    println!("violations: none (no allowlist configured)");

    let dot_path = out_dir.join("graph.dot");
    let mut dot = String::from("digraph workspace {\n");
    for (from, to) in edges {
        let _ = std::fmt::Write::write_fmt(
            &mut dot,
            format_args!("  \"{from}\" -> \"{to}\";\n"),
        );
    }
    dot.push_str("}\n");
    std::fs::write(&dot_path, dot).context("write graph.dot")?;
    println!("graph: {}", dot_path.display());
    Ok(())
}
