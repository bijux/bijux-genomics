use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use anyhow::{Context, Result};
use serde::Serialize;
use toml::Value;

#[derive(Debug, Serialize)]
pub struct CrateDependencyMapReport {
    pub schema_version: &'static str,
    pub workspace_manifest: String,
    pub output_path: String,
    pub crate_count: usize,
    pub edge_count: usize,
    pub crates: Vec<CrateDependencyNode>,
    pub edges: Vec<CrateDependencyEdge>,
}

#[derive(Debug, Serialize)]
pub struct CrateDependencyNode {
    pub crate_name: String,
    pub manifest_path: String,
    pub direct_workspace_dependencies: Vec<String>,
    pub direct_workspace_dependents: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct CrateDependencyEdge {
    pub from: String,
    pub to: String,
}

fn relative_display(path: &Path, root: &Path) -> String {
    path.strip_prefix(root).unwrap_or(path).display().to_string()
}

fn workspace_members(cwd: &Path) -> Result<Vec<(String, String)>> {
    let workspace_manifest_path = cwd.join("Cargo.toml");
    let manifest = std::fs::read_to_string(&workspace_manifest_path)
        .with_context(|| format!("read {}", workspace_manifest_path.display()))?;
    let value: Value = toml::from_str(&manifest)
        .with_context(|| format!("parse {}", workspace_manifest_path.display()))?;
    let members = value
        .get("workspace")
        .and_then(|workspace| workspace.get("members"))
        .and_then(Value::as_array)
        .context("workspace.members missing from root Cargo.toml")?;

    let mut resolved = Vec::new();
    for member in members {
        let member = member.as_str().context("workspace.members must contain only string paths")?;
        let manifest_path = cwd.join(member).join("Cargo.toml");
        let crate_manifest = std::fs::read_to_string(&manifest_path)
            .with_context(|| format!("read {}", manifest_path.display()))?;
        let crate_value: Value = toml::from_str(&crate_manifest)
            .with_context(|| format!("parse {}", manifest_path.display()))?;
        let crate_name = crate_value
            .get("package")
            .and_then(|package| package.get("name"))
            .and_then(Value::as_str)
            .with_context(|| format!("package.name missing from {}", manifest_path.display()))?;
        resolved.push((crate_name.to_string(), relative_display(&manifest_path, cwd)));
    }
    resolved.sort_by(|left, right| left.0.cmp(&right.0));
    Ok(resolved)
}

/// # Errors
/// Returns an error if the workspace crate graph cannot be resolved or written.
pub fn write_dependency_map(cwd: &Path, output_path: &Path) -> Result<CrateDependencyMapReport> {
    let members = workspace_members(cwd)?;
    let member_names = members
        .iter()
        .map(|(crate_name, _manifest_path)| crate_name.clone())
        .collect::<BTreeSet<_>>();
    let edges = bijux_dna_api::v1::api::workspace_edges().context("load workspace edges")?;

    let mut direct_deps = BTreeMap::<String, BTreeSet<String>>::new();
    let mut direct_dependents = BTreeMap::<String, BTreeSet<String>>::new();
    for crate_name in &member_names {
        direct_deps.insert(crate_name.clone(), BTreeSet::new());
        direct_dependents.insert(crate_name.clone(), BTreeSet::new());
    }

    let mut edge_rows = Vec::new();
    for (from, to) in edges {
        if !member_names.contains(&from) || !member_names.contains(&to) {
            continue;
        }
        direct_deps.entry(from.clone()).or_default().insert(to.clone());
        direct_dependents.entry(to.clone()).or_default().insert(from.clone());
        edge_rows.push(CrateDependencyEdge { from, to });
    }
    edge_rows
        .sort_by(|left, right| left.from.cmp(&right.from).then_with(|| left.to.cmp(&right.to)));

    let nodes = members
        .into_iter()
        .map(|(crate_name, manifest_path)| CrateDependencyNode {
            direct_workspace_dependencies: direct_deps
                .remove(&crate_name)
                .unwrap_or_default()
                .into_iter()
                .collect(),
            direct_workspace_dependents: direct_dependents
                .remove(&crate_name)
                .unwrap_or_default()
                .into_iter()
                .collect(),
            crate_name,
            manifest_path,
        })
        .collect::<Vec<_>>();

    let report = CrateDependencyMapReport {
        schema_version: "bijux.crates.dependency_map.v1",
        workspace_manifest: relative_display(&cwd.join("Cargo.toml"), cwd),
        output_path: relative_display(output_path, cwd),
        crate_count: nodes.len(),
        edge_count: edge_rows.len(),
        crates: nodes,
        edges: edge_rows,
    };

    if let Some(parent) = output_path.parent() {
        bijux_dna_infra::ensure_dir(parent)?;
    }
    bijux_dna_infra::atomic_write_json(output_path, &report)?;
    Ok(report)
}
