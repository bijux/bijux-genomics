use crate::diagnostics::BenchError;
use crate::model::{
    BenchmarkGraphNode, BenchmarkStageEdge, BenchmarkStageSpec, BenchmarkSuiteSpec,
};

use super::DeclaredStageNodes;
use crate::contract::suite::validate_edge_ports;

pub(super) fn validate_upstream_stage_references(
    suite: &BenchmarkSuiteSpec,
    declared_stage_nodes: &DeclaredStageNodes,
) -> Result<(), BenchError> {
    for stage in &suite.stages {
        let node_id = stage_node_id(stage);
        for upstream in &stage.upstream_stage_instance_ids {
            if !declared_stage_nodes.contains(upstream) {
                return Err(BenchError::InvalidPolicy(format!(
                    "suite stage {node_id} references unknown upstream stage node {upstream}"
                )));
            }
        }
    }
    Ok(())
}

pub(super) fn validate_explicit_edges(
    suite: &BenchmarkSuiteSpec,
    declared_graph_nodes: &std::collections::BTreeMap<String, BenchmarkGraphNode>,
) -> Result<(), BenchError> {
    let mut seen_edges = std::collections::BTreeSet::new();
    for edge in &suite.edges {
        validate_edge_identity(edge, declared_graph_nodes, &mut seen_edges)?;
        validate_edge_ports(edge, declared_graph_nodes)?;
    }
    Ok(())
}

fn validate_edge_identity(
    edge: &BenchmarkStageEdge,
    declared_graph_nodes: &std::collections::BTreeMap<String, BenchmarkGraphNode>,
    seen_edges: &mut std::collections::BTreeSet<(String, String, Option<String>, Option<String>)>,
) -> Result<(), BenchError> {
    if edge.from.trim().is_empty() || edge.to.trim().is_empty() {
        return Err(BenchError::InvalidPolicy(
            "suite edges must include non-empty from/to nodes".to_string(),
        ));
    }
    if edge.from == edge.to {
        return Err(BenchError::InvalidPolicy(format!(
            "suite edge {} -> {} must not reference itself",
            edge.from, edge.to
        )));
    }
    if !declared_graph_nodes.contains_key(&edge.from) {
        return Err(BenchError::InvalidPolicy(format!(
            "suite edge references unknown source node {}",
            edge.from
        )));
    }
    if !declared_graph_nodes.contains_key(&edge.to) {
        return Err(BenchError::InvalidPolicy(format!(
            "suite edge references unknown target node {}",
            edge.to
        )));
    }
    if !seen_edges.insert((
        edge.from.clone(),
        edge.to.clone(),
        edge.from_output_id.clone(),
        edge.to_input_id.clone(),
    )) {
        return Err(BenchError::InvalidPolicy(format!(
            "suite must not repeat edge {} -> {} with identical artifact bindings",
            edge.from, edge.to
        )));
    }
    if let Some(from_output_id) = edge.from_output_id.as_ref() {
        if from_output_id.trim().is_empty() {
            return Err(BenchError::InvalidPolicy(format!(
                "suite edge {} -> {} must not include blank from_output_id",
                edge.from, edge.to
            )));
        }
    }
    if let Some(to_input_id) = edge.to_input_id.as_ref() {
        if to_input_id.trim().is_empty() {
            return Err(BenchError::InvalidPolicy(format!(
                "suite edge {} -> {} must not include blank to_input_id",
                edge.from, edge.to
            )));
        }
    }
    Ok(())
}

fn stage_node_id(stage: &BenchmarkStageSpec) -> &str {
    stage.stage_instance_id.as_deref().unwrap_or(stage.stage.as_str())
}
