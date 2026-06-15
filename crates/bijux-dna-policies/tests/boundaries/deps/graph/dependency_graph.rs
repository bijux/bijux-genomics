#![allow(non_snake_case)]
#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use cargo_metadata::{DependencyKind, MetadataCommand};

fn workspace_root() -> PathBuf {
    bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"))
}

fn parse_boundary_contract() -> BTreeMap<String, BTreeSet<String>> {
    let root = workspace_root();
    let path = root.join("docs").join("10-architecture").join("BOUNDARY_MAP.md");
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("read {}: {err}", path.display()));
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
    bijux_dna_policies::policy_assert!(
        in_block && !lines.is_empty(),
        "missing executable boundaries block in {}",
        path.display()
    );
    let mut map = BTreeMap::new();
    for line in lines {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (name, deps) = line.split_once(':').unwrap_or_else(|| {
            bijux_dna_policies::policy_panic!("invalid boundaries line: {line}")
        });
        let deps = deps
            .split_whitespace()
            .filter(|dep| !dep.is_empty())
            .map(std::string::ToString::to_string)
            .collect::<BTreeSet<_>>();
        map.insert(name.trim().to_string(), deps);
    }
    map
}

#[test]
fn policy__boundaries__dependency_graph__dependency_dag_matches_boundaries() {
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
            if dep.kind != DependencyKind::Normal {
                continue;
            }
            if workspace_members.contains(&dep.name) {
                actual_deps.insert(dep.name.clone());
            }
        }
        let unexpected: BTreeSet<_> = actual_deps.difference(allowed_deps).cloned().collect();
        bijux_dna_policies::policy_assert!(
            unexpected.is_empty(),
            "{} depends on disallowed workspace crates: {:?}",
            package.name,
            unexpected
        );
    }
}

#[test]
fn policy__boundaries__dependency_graph__runner_has_no_engine_edge() {
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
        if pkg.name == "bijux-dna-runner" {
            runner_id = Some(pkg.id.clone());
        }
        if pkg.name == "bijux-dna-engine" {
            engine_id = Some(pkg.id.clone());
        }
    }
    let runner_id = runner_id.expect("bijux-dna-runner missing");
    let engine_id = engine_id.expect("bijux-dna-engine missing");
    let resolve = metadata.resolve.as_ref().expect("resolve graph missing");
    let node = resolve.nodes.iter().find(|node| node.id == runner_id).expect("runner node missing");
    let has_edge = node.deps.iter().any(|dep| dep.pkg == engine_id);
    bijux_dna_policies::policy_assert!(
        !has_edge,
        "{} must not depend on {}",
        id_to_name.get(&runner_id).map_or("bijux-dna-runner", String::as_str),
        id_to_name.get(&engine_id).map_or("bijux-dna-engine", String::as_str)
    );
}

#[test]
fn policy__boundaries__dependency_graph__engine_has_no_domain_or_planner_edges() {
    let root = workspace_root();
    let metadata = MetadataCommand::new()
        .manifest_path(root.join("Cargo.toml"))
        .exec()
        .expect("load cargo metadata");
    let mut engine_id = None;
    for pkg in &metadata.packages {
        if pkg.name == "bijux-dna-engine" {
            engine_id = Some(pkg.id.clone());
            break;
        }
    }
    let engine_id = engine_id.expect("bijux-dna-engine missing");
    let resolve = metadata.resolve.as_ref().expect("resolve graph missing");
    let node = resolve.nodes.iter().find(|node| node.id == engine_id).expect("engine node missing");
    let denylist = [
        "bijux-dna-domain-fastq",
        "bijux-dna-domain-bam",
        "bijux-dna-stages-fastq",
        "bijux-dna-stages-bam",
        "bijux-dna-planner-fastq",
        "bijux-dna-planner-bam",
    ];
    for dep in &node.deps {
        let dep_name = metadata
            .packages
            .iter()
            .find(|pkg| pkg.id == dep.pkg)
            .map_or("", |pkg| pkg.name.as_str());
        bijux_dna_policies::policy_assert!(
            !denylist.contains(&dep_name),
            "bijux-dna-engine must not depend on {dep_name}"
        );
    }
}

#[test]
fn policy__boundaries__dependency_graph__cli_depends_only_on_api() {
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
    let cli =
        metadata.packages.iter().find(|pkg| pkg.name == "bijux-dna").expect("bijux-dna missing");
    let allowed = BTreeSet::from([
        "bijux-dna-api".to_string(),
        "bijux-dna-analyze".to_string(),
        "bijux-dna-domain-compiler".to_string(),
        "bijux-dna-runtime".to_string(),
        "bijux-dna-infra".to_string(),
        "bijux-dna-domain-vcf".to_string(),
        "bijux-dna-stages-vcf".to_string(),
        "bijux-dna-db-ena".to_string(),
    ]);
    let actual: BTreeSet<String> = cli
        .dependencies
        .iter()
        .filter(|dep| dep.kind == DependencyKind::Normal)
        .filter(|dep| workspace_members.contains(&dep.name))
        .map(|dep| dep.name.clone())
        .collect();
    let unexpected: BTreeSet<_> = actual.difference(&allowed).cloned().collect();
    bijux_dna_policies::policy_assert!(
        unexpected.is_empty(),
        "bijux CLI depends on unexpected workspace crates: {:?}",
        unexpected
    );
}
