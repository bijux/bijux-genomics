use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use cargo_metadata::MetadataCommand;

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn parse_boundary_contract() -> BTreeMap<String, BTreeSet<String>> {
    let root = workspace_root();
    let path = root
        .join("crates")
        .join("bijux-core")
        .join("src")
        .join("boundaries.md");
    let content = std::fs::read_to_string(&path).expect("read boundaries.md");
    let mut lines = Vec::new();
    let mut in_block = false;
    for line in content.lines() {
        if line.trim() == "```boundaries" {
            in_block = true;
            continue;
        }
        if in_block && line.trim() == "```" {
            break;
        }
        if in_block {
            lines.push(line.trim().to_string());
        }
    }
    assert!(
        in_block && !lines.is_empty(),
        "missing executable boundaries block in {}",
        path.display()
    );
    let mut map = BTreeMap::new();
    for line in lines {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (name, deps) = line
            .split_once(':')
            .unwrap_or_else(|| panic!("invalid boundaries line: {line}"));
        let deps = deps
            .split_whitespace()
            .filter(|dep| !dep.is_empty())
            .map(|dep| dep.to_string())
            .collect::<BTreeSet<_>>();
        map.insert(name.trim().to_string(), deps);
    }
    map
}

#[test]
fn dependency_dag_matches_boundaries() {
    let root = workspace_root();
    let metadata = MetadataCommand::new()
        .manifest_path(root.join("Cargo.toml"))
        .exec()
        .expect("load cargo metadata");

    let workspace_members: BTreeSet<String> = metadata
        .workspace_members
        .iter()
        .filter_map(|id| metadata.packages.iter().find(|pkg| &pkg.id == id))
        .map(|pkg| pkg.name.clone())
        .collect();

    let allowed = parse_boundary_contract();
    for package in &metadata.packages {
        if !workspace_members.contains(&package.name) {
            continue;
        }
        let Some(allowed_deps) = allowed.get(&package.name) else {
            continue;
        };
        let mut actual_deps = BTreeSet::new();
        for dep in &package.dependencies {
            if workspace_members.contains(&dep.name) {
                actual_deps.insert(dep.name.clone());
            }
        }
        let unexpected: BTreeSet<_> = actual_deps.difference(allowed_deps).cloned().collect();
        assert!(
            unexpected.is_empty(),
            "{} depends on disallowed workspace crates: {:?}",
            package.name,
            unexpected
        );
    }
}

#[test]
fn runner_has_no_engine_edge() {
    let root = workspace_root();
    let metadata = MetadataCommand::new()
        .manifest_path(root.join("Cargo.toml"))
        .exec()
        .expect("load cargo metadata");
    let mut id_to_name = BTreeMap::new();
    for pkg in &metadata.packages {
        id_to_name.insert(pkg.id.clone(), pkg.name.clone());
    }
    let mut runner_id = None;
    let mut engine_id = None;
    for pkg in &metadata.packages {
        if pkg.name == "bijux-runner" {
            runner_id = Some(pkg.id.clone());
        }
        if pkg.name == "bijux-engine" {
            engine_id = Some(pkg.id.clone());
        }
    }
    let runner_id = runner_id.expect("bijux-runner missing");
    let engine_id = engine_id.expect("bijux-engine missing");
    let resolve = metadata.resolve.as_ref().expect("resolve graph missing");
    let node = resolve
        .nodes
        .iter()
        .find(|node| node.id == runner_id)
        .expect("runner node missing");
    let has_edge = node.deps.iter().any(|dep| dep.pkg == engine_id);
    assert!(
        !has_edge,
        "{} must not depend on {}",
        id_to_name
            .get(&runner_id)
            .map(String::as_str)
            .unwrap_or("bijux-runner"),
        id_to_name
            .get(&engine_id)
            .map(String::as_str)
            .unwrap_or("bijux-engine")
    );
}

#[test]
fn engine_has_no_domain_or_planner_edges() {
    let root = workspace_root();
    let metadata = MetadataCommand::new()
        .manifest_path(root.join("Cargo.toml"))
        .exec()
        .expect("load cargo metadata");
    let mut engine_id = None;
    for pkg in &metadata.packages {
        if pkg.name == "bijux-engine" {
            engine_id = Some(pkg.id.clone());
            break;
        }
    }
    let engine_id = engine_id.expect("bijux-engine missing");
    let resolve = metadata.resolve.as_ref().expect("resolve graph missing");
    let node = resolve
        .nodes
        .iter()
        .find(|node| node.id == engine_id)
        .expect("engine node missing");
    let denylist = [
        "bijux-domain-fastq",
        "bijux-domain-bam",
        "bijux-stages-fastq",
        "bijux-stages-bam",
        "bijux-planner-fastq",
        "bijux-planner-bam",
    ];
    for dep in &node.deps {
        let dep_name = metadata
            .packages
            .iter()
            .find(|pkg| pkg.id == dep.pkg)
            .map(|pkg| pkg.name.as_str())
            .unwrap_or("");
        assert!(
            !denylist.contains(&dep_name),
            "bijux-engine must not depend on {dep_name}"
        );
    }
}
