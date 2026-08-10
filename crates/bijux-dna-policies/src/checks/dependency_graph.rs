use std::collections::{BTreeMap, BTreeSet, VecDeque};

use anyhow::{bail, ensure, Result};
use serde::{Deserialize, Serialize};

pub const CI_DEPENDENCY_GRAPH_SCHEMA_V1: &str = "bijux.ci.dependency_graph.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CiDependencyGraph {
    pub schema: String,
    pub planner: CiPlannerPolicy,
    pub periodic_full_validation: CiPeriodicFullValidation,
    pub gate_families: Vec<CiGateFamily>,
    pub nodes: Vec<CiDependencyNode>,
    pub edges: Vec<CiDependencyEdge>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CiPlannerPolicy {
    pub path_match: String,
    pub dependency_propagation: String,
    pub unmatched_change: String,
    pub empty_plan: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CiPeriodicFullValidation {
    pub gate: String,
    pub cadence: String,
    pub validates_change_impact_planner: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CiGateFamily {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command_template: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CiDependencyNode {
    pub id: String,
    pub kind: CiNodeKind,
    pub paths: Vec<String>,
    pub gates: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CiNodeKind {
    Source,
    Crate,
    GeneratedAsset,
    Contract,
    Report,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CiDependencyEdge {
    pub consumer: String,
    pub dependency: String,
    pub kind: CiDependencyKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CiDependencyKind {
    Normal,
    Dev,
    Build,
    GeneratedBy,
    OwnedBy,
    DerivedFrom,
    ValidatedBy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChangeImpactPlan {
    pub changed_paths: Vec<String>,
    pub directly_matched_nodes: Vec<String>,
    pub affected_nodes: Vec<String>,
    pub gates: Vec<String>,
    pub used_full_fallback: bool,
}

/// Validate the declarative CI graph and its fail-closed planner policy.
///
/// # Errors
/// Returns an error when graph identities, paths, gates, or dependency references are invalid.
pub fn validate_dependency_graph(graph: &CiDependencyGraph) -> Result<()> {
    ensure!(
        graph.schema == CI_DEPENDENCY_GRAPH_SCHEMA_V1,
        "unsupported CI dependency graph schema"
    );
    ensure!(
        graph.planner.path_match == "longest_declared_path",
        "path matching must be longest_declared_path"
    );
    ensure!(
        graph.planner.dependency_propagation == "reverse_dependents",
        "dependency propagation must be reverse_dependents"
    );
    ensure!(graph.planner.unmatched_change == "ci-full", "unmatched changes must select ci-full");
    ensure!(graph.planner.empty_plan == "ci-full", "empty plans must select ci-full");
    ensure!(
        graph.periodic_full_validation.gate == "ci-full",
        "periodic validation must run ci-full"
    );
    ensure!(
        graph.periodic_full_validation.validates_change_impact_planner,
        "periodic full validation must validate the planner"
    );
    ensure!(
        !graph.periodic_full_validation.cadence.trim().is_empty(),
        "periodic full validation cadence is required"
    );

    let families = graph.gate_families.iter().map(|gate| gate.id.as_str()).collect::<BTreeSet<_>>();
    ensure!(families.len() == graph.gate_families.len(), "gate family identifiers must be unique");
    ensure!(families.contains("ci-full"), "ci-full gate family is required");
    for gate in &graph.gate_families {
        ensure!(!gate.id.trim().is_empty(), "gate family identifier cannot be empty");
        ensure!(
            gate.command.is_some() ^ gate.command_template.is_some(),
            "gate family {} must declare exactly one command form",
            gate.id
        );
        let command = gate.command.as_deref().or(gate.command_template.as_deref()).unwrap_or("");
        ensure!(!command.trim().is_empty(), "gate family {} has an empty command", gate.id);
        if gate.id == "crate-tests" {
            ensure!(
                gate.command_template.as_deref().is_some_and(|value| value.contains("{crate}")),
                "crate-tests must declare a command template containing {{crate}}"
            );
        }
    }

    let mut nodes = BTreeMap::new();
    let mut paths = BTreeSet::new();
    let mut kinds = BTreeSet::new();
    for node in &graph.nodes {
        ensure!(!node.id.trim().is_empty(), "node identifier cannot be empty");
        ensure!(nodes.insert(node.id.as_str(), node).is_none(), "duplicate node {}", node.id);
        kinds.insert(node.kind);
        ensure!(!node.paths.is_empty(), "node {} has no governed paths", node.id);
        ensure!(!node.gates.is_empty(), "node {} has no gates", node.id);
        for path in &node.paths {
            validate_repo_path(path)?;
            ensure!(paths.insert(path.as_str()), "duplicate governed path {path}");
        }
        for gate in &node.gates {
            let family = gate.split_once(':').map_or(gate.as_str(), |(family, _)| family);
            ensure!(
                families.contains(family),
                "node {} references unknown gate family {family}",
                node.id
            );
        }
    }
    for required in [
        CiNodeKind::Source,
        CiNodeKind::Crate,
        CiNodeKind::GeneratedAsset,
        CiNodeKind::Contract,
        CiNodeKind::Report,
    ] {
        ensure!(
            kinds.contains(&required),
            "dependency graph lacks required node kind {required:?}"
        );
    }

    let mut edges = BTreeSet::new();
    for edge in &graph.edges {
        ensure!(
            nodes.contains_key(edge.consumer.as_str()),
            "unknown edge consumer {}",
            edge.consumer
        );
        ensure!(
            nodes.contains_key(edge.dependency.as_str()),
            "unknown edge dependency {}",
            edge.dependency
        );
        ensure!(edge.consumer != edge.dependency, "self dependency for {}", edge.consumer);
        ensure!(
            edges.insert(edge),
            "duplicate dependency edge {} -> {}",
            edge.consumer,
            edge.dependency
        );
    }
    Ok(())
}

/// Compute all direct and reverse-dependent gates for a set of changed repository paths.
///
/// # Errors
/// Returns an error when the graph is invalid or a changed path is not portable and relative.
pub fn plan_change_impact(
    graph: &CiDependencyGraph,
    changed_paths: &[String],
) -> Result<ChangeImpactPlan> {
    validate_dependency_graph(graph)?;
    let mut direct = BTreeSet::new();
    let mut used_full_fallback = changed_paths.is_empty();
    for changed in changed_paths {
        validate_repo_path(changed)?;
        let matches = graph
            .nodes
            .iter()
            .filter(|node| node.paths.iter().any(|selector| path_matches(selector, changed)));
        let longest = matches.max_by_key(|node| {
            node.paths
                .iter()
                .filter(|selector| path_matches(selector, changed))
                .map(String::len)
                .max()
                .unwrap_or_default()
        });
        if let Some(node) = longest {
            direct.insert(node.id.clone());
        } else {
            used_full_fallback = true;
        }
    }

    let mut affected = direct.clone();
    let mut frontier = direct.iter().cloned().collect::<VecDeque<_>>();
    while let Some(dependency) = frontier.pop_front() {
        for edge in graph.edges.iter().filter(|edge| edge.dependency == dependency) {
            if affected.insert(edge.consumer.clone()) {
                frontier.push_back(edge.consumer.clone());
            }
        }
    }
    let node_by_id =
        graph.nodes.iter().map(|node| (node.id.as_str(), node)).collect::<BTreeMap<_, _>>();
    let mut gates = affected
        .iter()
        .flat_map(|id| node_by_id[id.as_str()].gates.iter().cloned())
        .collect::<BTreeSet<_>>();
    if used_full_fallback || gates.is_empty() {
        gates.insert("ci-full".to_string());
        used_full_fallback = true;
    }
    Ok(ChangeImpactPlan {
        changed_paths: changed_paths.to_vec(),
        directly_matched_nodes: direct.into_iter().collect(),
        affected_nodes: affected.into_iter().collect(),
        gates: gates.into_iter().collect(),
        used_full_fallback,
    })
}

fn validate_repo_path(path: &str) -> Result<()> {
    ensure!(!path.is_empty(), "repository path cannot be empty");
    ensure!(
        !path.starts_with('/') && !path.contains('\\'),
        "repository path must be portable and relative: {path}"
    );
    if path.split('/').any(|part| part == "..") {
        bail!("repository path cannot traverse parents: {path}");
    }
    Ok(())
}

fn path_matches(selector: &str, changed: &str) -> bool {
    if selector.ends_with('/') {
        changed.starts_with(selector)
    } else {
        changed == selector
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn graph() -> Result<CiDependencyGraph> {
        Ok(serde_json::from_str(include_str!("../../../../configs/ci/dependency-graph.json"))?)
    }

    #[test]
    fn repository_graph_is_valid_and_complete_by_kind() -> Result<()> {
        validate_dependency_graph(&graph()?)
    }

    #[test]
    fn crate_change_propagates_to_reverse_dependents() -> Result<()> {
        let plan =
            plan_change_impact(&graph()?, &["crates/bijux-dna-core/src/lib.rs".to_string()])?;
        assert!(plan.affected_nodes.contains(&"crate:bijux-dna-core".to_string()));
        assert!(plan.affected_nodes.contains(&"crate:bijux-dna".to_string()));
        assert!(plan.gates.contains(&"crate-tests:bijux-dna".to_string()));
        assert!(!plan.used_full_fallback);
        Ok(())
    }

    #[test]
    fn unmatched_and_empty_changes_fail_closed_to_full_ci() -> Result<()> {
        for paths in [vec!["docs/unclassified.md".to_string()], Vec::new()] {
            let plan = plan_change_impact(&graph()?, &paths)?;
            assert_eq!(plan.gates, vec!["ci-full"]);
            assert!(plan.used_full_fallback);
        }
        Ok(())
    }

    #[test]
    fn generated_schema_changes_select_drift_and_domain_contract_gates() -> Result<()> {
        let plan = plan_change_impact(
            &graph()?,
            &["benchmarks/contracts/schemas/run-index.schema.json".to_string()],
        )?;
        assert_eq!(plan.directly_matched_nodes, vec!["generated:benchmark-schemas".to_string()]);
        assert!(plan.gates.contains(&"generated-configs".to_string()));
        assert!(plan.gates.contains(&"benchmark-contracts".to_string()));
        assert!(!plan.used_full_fallback);
        Ok(())
    }
}
