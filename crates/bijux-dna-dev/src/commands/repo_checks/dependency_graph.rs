use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use anyhow::{ensure, Context, Result};
use bijux_dna_policies::{
    validate_dependency_graph, CiDependencyEdge, CiDependencyGraph, CiDependencyKind, CiNodeKind,
};
use serde_json::Value;

use crate::commands::command_support::{pass, run_command};
use crate::model::check::{CheckDefinition, CheckOutcome};
use crate::runtime::workspace::Workspace;

const GRAPH_PATH: &str = "configs/ci/dependency-graph.json";
const CARGO_MAKE_PATH: &str = "makes/cargo.mk";
const FAST_GROUP_PATH: &str = "configs/ci/fast-groups.json";
const GRAPH_CHECK_COMMAND: &str =
    "cargo run -q -p bijux-dna-dev -- checks run check-ci-dependency-graph";

pub(crate) fn check_ci_dependency_graph(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    let raw = std::fs::read_to_string(workspace.path(GRAPH_PATH))
        .with_context(|| format!("read {GRAPH_PATH}"))?;
    let graph: CiDependencyGraph =
        serde_json::from_str(&raw).with_context(|| format!("parse {GRAPH_PATH}"))?;
    validate_dependency_graph(&graph)?;

    let metadata = cargo_metadata(workspace)?;
    let expected_crates = workspace_crates(workspace, &metadata)?;
    let declared_crates = graph
        .nodes
        .iter()
        .filter(|node| node.kind == CiNodeKind::Crate)
        .map(|node| (node.id.clone(), node.paths.clone()))
        .collect::<BTreeMap<_, _>>();
    ensure!(
        declared_crates == expected_crates,
        "crate nodes differ from cargo metadata\nexpected={expected_crates:#?}\ndeclared={declared_crates:#?}"
    );

    let expected_edges = workspace_dependency_edges(&metadata)?;
    let declared_edges = graph
        .edges
        .iter()
        .filter(|edge| {
            matches!(
                edge.kind,
                CiDependencyKind::Normal | CiDependencyKind::Dev | CiDependencyKind::Build
            )
        })
        .cloned()
        .collect::<BTreeSet<_>>();
    ensure!(
        declared_edges == expected_edges,
        "crate dependency edges differ from cargo metadata\nmissing={:#?}\nstale={:#?}",
        expected_edges.difference(&declared_edges).collect::<Vec<_>>(),
        declared_edges.difference(&expected_edges).collect::<Vec<_>>()
    );
    let cargo_make = std::fs::read_to_string(workspace.path(CARGO_MAKE_PATH))
        .with_context(|| format!("read {CARGO_MAKE_PATH}"))?;
    let fast_body = make_target_body(&cargo_make, "_ci-fast")?;
    ensure!(
        fast_body.contains("tooling run ci-fast"),
        "_ci-fast must delegate to the native DAG supervisor"
    );
    let fast_groups = std::fs::read_to_string(workspace.path(FAST_GROUP_PATH))
        .with_context(|| format!("read {FAST_GROUP_PATH}"))?;
    ensure!(
        fast_groups.contains("_ci-authority-fast"),
        "ci-fast groups must include the dependency-graph target"
    );
    let slow_body = make_target_body(&cargo_make, "_ci-slow")?;
    ensure!(
        slow_body.contains(GRAPH_CHECK_COMMAND),
        "_ci-slow must validate the CI dependency graph"
    );

    pass(
        check,
        format!(
            "validated {} nodes and {} edges against {} workspace crates; unmatched changes fail closed to ci-full",
            graph.nodes.len(),
            graph.edges.len(),
            expected_crates.len()
        ),
    )
}

fn make_target_body(source: &str, target: &str) -> Result<String> {
    let marker = format!("{target}:");
    let mut lines = source.lines();
    ensure!(lines.by_ref().any(|line| line.starts_with(&marker)), "missing Make target {target}");
    let body = lines
        .take_while(|line| line.starts_with('\t') || line.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    ensure!(!body.trim().is_empty(), "Make target {target} has no recipe");
    Ok(body)
}

fn cargo_metadata(workspace: &Workspace) -> Result<Value> {
    let output = run_command(
        workspace,
        "cargo",
        &["metadata", "--format-version", "1", "--locked", "--no-deps"],
    )?;
    ensure!(output.status.success(), "cargo metadata failed");
    serde_json::from_slice(&output.stdout).context("parse cargo metadata output")
}

fn workspace_crates(
    workspace: &Workspace,
    metadata: &Value,
) -> Result<BTreeMap<String, Vec<String>>> {
    let member_ids =
        string_array(metadata, "workspace_members")?.into_iter().collect::<BTreeSet<_>>();
    let mut crates = BTreeMap::new();
    for package in value_array(metadata, "packages")? {
        let id = string_field(package, "id")?;
        if !member_ids.contains(id) {
            continue;
        }
        let name = string_field(package, "name")?;
        let manifest = Path::new(string_field(package, "manifest_path")?);
        let crate_root = manifest.parent().context("workspace manifest has no parent")?;
        let relative = workspace.rel(crate_root).to_string_lossy().replace('\\', "/");
        ensure!(relative.starts_with("crates/"), "workspace crate is outside crates/: {relative}");
        crates.insert(format!("crate:{name}"), vec![format!("{relative}/")]);
    }
    Ok(crates)
}

fn workspace_dependency_edges(metadata: &Value) -> Result<BTreeSet<CiDependencyEdge>> {
    let member_ids =
        string_array(metadata, "workspace_members")?.into_iter().collect::<BTreeSet<_>>();
    let packages = value_array(metadata, "packages")?;
    let member_names = packages
        .iter()
        .filter_map(|package| {
            let id = package.get("id")?.as_str()?;
            if !member_ids.contains(id) {
                return None;
            }
            package.get("name")?.as_str().map(str::to_string)
        })
        .collect::<BTreeSet<_>>();
    let mut edges = BTreeSet::new();
    for package in packages {
        if !member_ids.contains(string_field(package, "id")?) {
            continue;
        }
        let consumer = format!("crate:{}", string_field(package, "name")?);
        for dependency in value_array(package, "dependencies")? {
            if dependency.get("path").is_none_or(Value::is_null) {
                continue;
            }
            let dependency_name = string_field(dependency, "name")?;
            ensure!(
                member_names.contains(dependency_name),
                "path dependency {dependency_name} is not a workspace member"
            );
            let kind = match dependency.get("kind").and_then(Value::as_str) {
                None | Some("normal") => CiDependencyKind::Normal,
                Some("dev") => CiDependencyKind::Dev,
                Some("build") => CiDependencyKind::Build,
                Some(other) => anyhow::bail!("unsupported cargo dependency kind {other}"),
            };
            edges.insert(CiDependencyEdge {
                consumer: consumer.clone(),
                dependency: format!("crate:{dependency_name}"),
                kind,
            });
        }
    }
    Ok(edges)
}

fn value_array<'a>(value: &'a Value, field: &str) -> Result<&'a Vec<Value>> {
    value
        .get(field)
        .and_then(Value::as_array)
        .with_context(|| format!("metadata field {field} is not an array"))
}

fn string_array<'a>(value: &'a Value, field: &str) -> Result<Vec<&'a str>> {
    value_array(value, field)?
        .iter()
        .map(|item| {
            item.as_str().with_context(|| format!("metadata field {field} contains a non-string"))
        })
        .collect()
}

fn string_field<'a>(value: &'a Value, field: &str) -> Result<&'a str> {
    value
        .get(field)
        .and_then(Value::as_str)
        .with_context(|| format!("metadata field {field} is not a string"))
}
