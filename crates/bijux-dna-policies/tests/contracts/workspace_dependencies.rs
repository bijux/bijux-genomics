#![allow(non_snake_case)]
use bijux_dna_policies::analyze_workspace_dependencies;
use std::collections::{BTreeMap, BTreeSet};
fn graph(nodes: &[&str], edges: &[(&str, &str)]) -> BTreeMap<String, BTreeSet<String>> {
    let mut graph = nodes
        .iter()
        .map(|node| ((*node).to_string(), BTreeSet::<String>::new()))
        .collect::<BTreeMap<_, _>>();
    for (from, to) in edges {
        graph.entry((*from).to_string()).or_default().insert((*to).to_string());
    }
    graph
}

#[test]
fn policy__contracts__workspace_dependencies__cycle_components_detects_multi_crate_cycle(
) -> anyhow::Result<()> {
    let graph = graph(
        &["bijux-dna-analyze", "bijux-dna-bench", "bijux-dna-bench-model"],
        &[
            ("bijux-dna-analyze", "bijux-dna-bench"),
            ("bijux-dna-bench", "bijux-dna-bench-model"),
            ("bijux-dna-bench-model", "bijux-dna-analyze"),
        ],
    );

    assert_eq!(
        analyze_workspace_dependencies(&graph)?.cycles,
        vec![vec![
            "bijux-dna-analyze".to_string(),
            "bijux-dna-bench".to_string(),
            "bijux-dna-bench-model".to_string(),
        ]]
    );
    Ok(())
}

#[test]
fn policy__contracts__workspace_dependencies__topological_dependency_order_lists_dependencies_before_dependents(
) -> anyhow::Result<()> {
    let graph = graph(
        &["bijux-dna", "bijux-dna-api", "bijux-dna-core"],
        &[("bijux-dna", "bijux-dna-api"), ("bijux-dna-api", "bijux-dna-core")],
    );

    assert_eq!(
        analyze_workspace_dependencies(&graph)?.dependency_order,
        vec!["bijux-dna-core".to_string(), "bijux-dna-api".to_string(), "bijux-dna".to_string(),]
    );
    Ok(())
}

#[test]
fn policy__contracts__workspace_dependencies__workspace_dependencies_reject_unknown_nodes_and_handle_disconnected_cycles(
) -> anyhow::Result<()> {
    let unknown = graph(&["a"], &[("a", "missing")]);
    assert!(analyze_workspace_dependencies(&unknown).is_err());
    assert!(analyze_workspace_dependencies(&graph(&[" "], &[])).is_err());
    let result = analyze_workspace_dependencies(&graph(
        &["a", "b", "c", "d"],
        &[("a", "a"), ("b", "c"), ("c", "b")],
    ))?;
    assert_eq!(result.cycles, vec![vec!["a".to_string()], vec!["b".to_string(), "c".to_string()]]);
    assert!(result.dependency_order.is_empty());
    let result =
        analyze_workspace_dependencies(&graph(&["a", "b", "c", "d"], &[("a", "c"), ("b", "c")]))?;
    assert!(result.cycles.is_empty());
    assert_eq!(result.dependency_order, vec!["c", "a", "b", "d"]);
    Ok(())
}

#[test]
fn policy__contracts__workspace_dependencies__workspace_dependencies_handle_long_chains_without_recursive_traversal(
) -> anyhow::Result<()> {
    let mut graph = BTreeMap::new();
    for index in 0..4096 {
        let dependencies = if index == 0 {
            BTreeSet::new()
        } else {
            BTreeSet::from([format!("node-{:04}", index - 1)])
        };
        graph.insert(format!("node-{index:04}"), dependencies);
    }
    let result = analyze_workspace_dependencies(&graph)?;
    assert!(result.cycles.is_empty());
    assert_eq!(result.dependency_order, graph.keys().cloned().collect::<Vec<_>>());
    Ok(())
}
