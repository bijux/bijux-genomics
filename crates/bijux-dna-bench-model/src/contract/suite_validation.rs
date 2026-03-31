//! Owner: bijux-dna-bench
//! Benchmark suite contract validation entrypoint.

use crate::error::BenchError;
use crate::model::{
    BenchmarkGraphNode, BenchmarkStageEdge, BenchmarkStageSpec, BenchmarkSuiteSpec,
};

use super::edge_validation::validate_edge_ports;
use super::param_binding_validation::validate_stage_param_bindings;
use super::stage_governance::{
    ensure_supported_stage, planner_owned_graph_stage, validate_stage_tools,
};
use super::suite_analysis::validate_suite_analysis_requirements;
use super::suite_diversity::validate_suite_diversity;
use super::suite_graph::{declared_graph_nodes, validate_suite_dag};
use super::SUITE_SCHEMA_V1;

type DeclaredStageNodes = std::collections::BTreeSet<String>;

/// # Errors
/// Returns an error if the suite spec violates required fields.
pub fn validate_suite(suite: &BenchmarkSuiteSpec) -> Result<(), BenchError> {
    validate_schema_version(suite)?;
    validate_suite_diversity(suite)?;
    let declared_stage_nodes = validate_stage_definitions(suite)?;
    let declared_graph_nodes = declared_graph_nodes(suite);
    validate_upstream_stage_references(suite, &declared_stage_nodes)?;
    validate_explicit_edges(suite, &declared_graph_nodes)?;
    validate_suite_analysis_requirements(suite)?;
    validate_suite_dag(suite)?;
    Ok(())
}

fn validate_schema_version(suite: &BenchmarkSuiteSpec) -> Result<(), BenchError> {
    if suite.schema_version == SUITE_SCHEMA_V1 {
        return Ok(());
    }
    Err(BenchError::InvalidPolicy(format!(
        "suite schema mismatch: {}",
        suite.schema_version
    )))
}

fn validate_stage_definitions(
    suite: &BenchmarkSuiteSpec,
) -> Result<DeclaredStageNodes, BenchError> {
    let mut seen_stage_nodes = std::collections::BTreeSet::new();
    let mut seen_stage_tool_nodes = std::collections::BTreeSet::new();
    let mut declared_stage_nodes = DeclaredStageNodes::new();
    for stage in &suite.stages {
        validate_stage_identity(stage, &mut seen_stage_nodes, &mut declared_stage_nodes)?;
        validate_stage_tool_contracts(stage, &mut seen_stage_tool_nodes)?;
        validate_stage_param_contracts(stage)?;
        validate_stage_upstream_contracts(stage)?;
    }
    Ok(declared_stage_nodes)
}

fn validate_stage_identity(
    stage: &BenchmarkStageSpec,
    seen_stage_nodes: &mut std::collections::BTreeSet<String>,
    declared_stage_nodes: &mut DeclaredStageNodes,
) -> Result<(), BenchError> {
    if stage.stage.trim().is_empty() {
        return Err(BenchError::InvalidPolicy(
            "suite stages must include non-empty stage ids".to_string(),
        ));
    }
    ensure_supported_stage(&stage.stage)?;
    let node_id = stage_node_id(stage);
    if node_id.trim().is_empty() {
        return Err(BenchError::InvalidPolicy(format!(
            "suite stage {} must not include blank stage_instance_id",
            stage.stage
        )));
    }
    if !seen_stage_nodes.insert(node_id.to_string()) {
        return Err(BenchError::InvalidPolicy(format!(
            "suite must not repeat stage node {node_id}"
        )));
    }
    declared_stage_nodes.insert(node_id.to_string());
    Ok(())
}

fn validate_stage_tool_contracts(
    stage: &BenchmarkStageSpec,
    seen_stage_tool_nodes: &mut std::collections::BTreeSet<String>,
) -> Result<(), BenchError> {
    if stage.tools.is_empty() && !planner_owned_graph_stage(&stage.stage) {
        return Err(BenchError::InvalidPolicy(format!(
            "suite stage {} must include at least one tool",
            stage.stage
        )));
    }
    if planner_owned_graph_stage(&stage.stage) && !stage.tools.is_empty() {
        return Err(BenchError::InvalidPolicy(format!(
            "suite planner-owned stage {} must not declare tool bindings",
            stage.stage
        )));
    }
    let mut seen_tools = std::collections::BTreeSet::new();
    for tool in &stage.tools {
        if tool.trim().is_empty() {
            return Err(BenchError::InvalidPolicy(format!(
                "suite stage {} must not include blank tool ids",
                stage.stage
            )));
        }
        if !seen_tools.insert(tool.as_str()) {
            return Err(BenchError::InvalidPolicy(format!(
                "suite stage {} must not repeat tool {}",
                stage.stage, tool
            )));
        }
        let tool_node_id = stage.tool_node_id(tool);
        if !seen_stage_tool_nodes.insert(tool_node_id.clone()) {
            return Err(BenchError::InvalidPolicy(format!(
                "suite must not repeat stage-tool node {tool_node_id}"
            )));
        }
    }
    validate_stage_tools(&stage.stage, &stage.tools)
}

fn validate_stage_param_contracts(stage: &BenchmarkStageSpec) -> Result<(), BenchError> {
    let mut seen_params = std::collections::BTreeSet::new();
    for params in &stage.params {
        if params.trim().is_empty() {
            return Err(BenchError::InvalidPolicy(format!(
                "suite stage {} must not include blank params entries",
                stage.stage
            )));
        }
        if !seen_params.insert(params.as_str()) {
            return Err(BenchError::InvalidPolicy(format!(
                "suite stage {} must not repeat params entry {}",
                stage.stage, params
            )));
        }
    }
    validate_stage_param_bindings(stage)
}

fn validate_stage_upstream_contracts(stage: &BenchmarkStageSpec) -> Result<(), BenchError> {
    let node_id = stage_node_id(stage);
    for upstream in &stage.upstream_stage_instance_ids {
        if upstream.trim().is_empty() {
            return Err(BenchError::InvalidPolicy(format!(
                "suite stage {} must not include blank upstream_stage_instance_ids",
                stage.stage
            )));
        }
        if upstream == node_id {
            return Err(BenchError::InvalidPolicy(format!(
                "suite stage {node_id} must not reference itself as an upstream stage"
            )));
        }
    }
    Ok(())
}

fn validate_upstream_stage_references(
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

fn validate_explicit_edges(
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
    stage
        .stage_instance_id
        .as_deref()
        .unwrap_or(stage.stage.as_str())
}
