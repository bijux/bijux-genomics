//! Pure analysis of the admitted workspace dependency graph.
use anyhow::{ensure, Context, Result};
use std::collections::{BTreeMap, BTreeSet};

/// Deterministic cycle membership and dependency-first ordering of a closed graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceDependencyAnalysis {
    pub cycles: Vec<Vec<String>>,
    /// Empty for cyclic graphs; otherwise includes every admitted node exactly once.
    pub dependency_order: Vec<String>,
}

/// Analyze declared nodes and edges without source reads or process execution.
/// # Errors
/// Rejects references to undeclared nodes and inconsistent traversal state.
pub fn analyze_workspace_dependencies(
    graph: &BTreeMap<String, BTreeSet<String>>,
) -> Result<WorkspaceDependencyAnalysis> {
    for (node, dependencies) in graph {
        ensure!(!node.trim().is_empty(), "workspace dependency node is empty");
        for dependency in dependencies {
            ensure!(
                graph.contains_key(dependency),
                "workspace dependency {node} references undeclared node {dependency}"
            );
        }
    }
    let mut dependents: BTreeMap<&str, Vec<&str>> =
        graph.keys().map(|node| (node.as_str(), Vec::new())).collect();
    for (node, dependencies) in graph {
        for dependency in dependencies {
            dependents
                .get_mut(dependency.as_str())
                .context("admitted dependency disappeared")?
                .push(node);
        }
    }
    let components = components(graph, &dependents)?;
    let cycles = components
        .into_iter()
        .filter(|component| {
            component.len() > 1
                || component.first().is_some_and(|node| {
                    graph.get(node).is_some_and(|dependencies| dependencies.contains(node))
                })
        })
        .collect::<Vec<_>>();
    let dependency_order =
        if cycles.is_empty() { dependency_order(graph, &dependents)? } else { Vec::new() };
    Ok(WorkspaceDependencyAnalysis { cycles, dependency_order })
}

fn components(
    graph: &BTreeMap<String, BTreeSet<String>>,
    dependents: &BTreeMap<&str, Vec<&str>>,
) -> Result<Vec<Vec<String>>> {
    let mut seen = BTreeSet::new();
    let mut finished = Vec::with_capacity(graph.len());
    for node in graph.keys() {
        let mut stack = vec![(node.as_str(), false)];
        while let Some((node, expanded)) = stack.pop() {
            if expanded {
                finished.push(node);
            } else if seen.insert(node) {
                stack.push((node, true));
                for dependency in
                    graph.get(node).context("admitted traversal node disappeared")?.iter().rev()
                {
                    if !seen.contains(dependency.as_str()) {
                        stack.push((dependency.as_str(), false));
                    }
                }
            }
        }
    }
    seen.clear();
    let mut components = Vec::new();
    for start in finished.into_iter().rev() {
        if seen.contains(start) {
            continue;
        }
        let mut stack = vec![start];
        let mut component = Vec::new();
        while let Some(node) = stack.pop() {
            if seen.insert(node) {
                component.push(node.to_string());
                stack.extend(
                    dependents
                        .get(node)
                        .context("admitted reverse node disappeared")?
                        .iter()
                        .copied(),
                );
            }
        }
        component.sort();
        components.push(component);
    }
    components.sort();
    Ok(components)
}

fn dependency_order(
    graph: &BTreeMap<String, BTreeSet<String>>,
    dependents: &BTreeMap<&str, Vec<&str>>,
) -> Result<Vec<String>> {
    let mut remaining = graph
        .iter()
        .map(|(node, dependencies)| (node.as_str(), dependencies.len()))
        .collect::<BTreeMap<_, _>>();
    let mut ready = remaining
        .iter()
        .filter_map(|(node, count)| (*count == 0).then_some(*node))
        .collect::<BTreeSet<_>>();
    let mut order = Vec::with_capacity(graph.len());
    while let Some(node) = ready.pop_first() {
        order.push(node.to_string());
        for dependent in
            dependents.get(node).context("admitted dependency-order node disappeared")?
        {
            let count = remaining.get_mut(dependent).context("admitted dependent disappeared")?;
            *count = count.checked_sub(1).context("dependency degree underflow")?;
            if *count == 0 {
                ready.insert(*dependent);
            }
        }
    }
    ensure!(order.len() == graph.len(), "acyclic dependency order is incomplete");
    Ok(order)
}
