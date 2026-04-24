//! Owner: bijux-dna-bench
//! Graph support for benchmark suite contract validation.

use crate::diagnostics::BenchError;
use crate::model::{BenchmarkGraphNode, BenchmarkSuiteSpec};

pub(crate) fn declared_graph_nodes(
    suite: &BenchmarkSuiteSpec,
) -> std::collections::BTreeMap<String, BenchmarkGraphNode> {
    suite.graph_nodes().into_iter().map(|node| (node.node_id.clone(), node)).collect()
}

pub(crate) fn validate_suite_dag(suite: &BenchmarkSuiteSpec) -> Result<(), BenchError> {
    let mut incoming = std::collections::BTreeMap::new();
    let mut outgoing = std::collections::BTreeMap::<String, Vec<String>>::new();
    for node_id in declared_graph_nodes(suite).into_keys() {
        incoming.entry(node_id).or_insert(0usize);
    }
    for stage in &suite.stages {
        let node_id =
            stage.stage_instance_id.as_deref().unwrap_or(stage.stage.as_str()).to_string();
        for upstream in &stage.upstream_stage_instance_ids {
            outgoing.entry(upstream.clone()).or_default().push(node_id.clone());
            *incoming.entry(node_id.clone()).or_insert(0) += 1;
        }
    }
    for edge in &suite.edges {
        outgoing.entry(edge.from.clone()).or_default().push(edge.to.clone());
        *incoming.entry(edge.to.clone()).or_insert(0) += 1;
    }
    let mut ready = incoming
        .iter()
        .filter_map(|(node, count)| if *count == 0 { Some(node.clone()) } else { None })
        .collect::<Vec<_>>();
    let mut visited = 0usize;
    while let Some(node) = ready.pop() {
        visited += 1;
        if let Some(children) = outgoing.get(&node) {
            for child in children {
                if let Some(count) = incoming.get_mut(child) {
                    *count -= 1;
                    if *count == 0 {
                        ready.push(child.clone());
                    }
                }
            }
        }
    }
    if visited != incoming.len() {
        return Err(BenchError::InvalidPolicy("suite graph must be acyclic".to_string()));
    }
    Ok(())
}
