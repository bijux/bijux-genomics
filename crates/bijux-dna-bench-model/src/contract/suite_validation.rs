//! Owner: bijux-dna-bench
//! Benchmark suite contract validation entrypoint.

use crate::error::BenchError;
use crate::model::BenchmarkSuiteSpec;

use super::edge_validation::validate_edge_ports;
use super::param_binding_validation::validate_stage_param_bindings;
use super::stage_governance::{
    ensure_supported_stage, planner_owned_graph_stage, validate_stage_tools,
};
use super::suite_analysis::validate_suite_analysis_requirements;
use super::suite_diversity::validate_suite_diversity;
use super::suite_graph::{declared_graph_nodes, validate_suite_dag};
use super::SUITE_SCHEMA_V1;

/// # Errors
/// Returns an error if the suite spec violates required fields.
pub fn validate_suite(suite: &BenchmarkSuiteSpec) -> Result<(), BenchError> {
    if suite.schema_version != SUITE_SCHEMA_V1 {
        return Err(BenchError::InvalidPolicy(format!(
            "suite schema mismatch: {}",
            suite.schema_version
        )));
    }
    validate_suite_diversity(suite)?;
    let mut seen_stage_nodes = std::collections::BTreeSet::new();
    let mut seen_stage_tool_nodes = std::collections::BTreeSet::new();
    let mut declared_stage_nodes = Vec::new();
    for stage in &suite.stages {
        if stage.stage.trim().is_empty() {
            return Err(BenchError::InvalidPolicy(
                "suite stages must include non-empty stage ids".to_string(),
            ));
        }
        ensure_supported_stage(&stage.stage)?;
        let node_id = stage
            .stage_instance_id
            .as_deref()
            .unwrap_or(stage.stage.as_str());
        if node_id.trim().is_empty() {
            return Err(BenchError::InvalidPolicy(format!(
                "suite stage {} must not include blank stage_instance_id",
                stage.stage
            )));
        }
        if !seen_stage_nodes.insert(node_id) {
            return Err(BenchError::InvalidPolicy(format!(
                "suite must not repeat stage node {node_id}"
            )));
        }
        declared_stage_nodes.push(node_id.to_string());
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
        validate_stage_tools(&stage.stage, &stage.tools)?;
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
        validate_stage_param_bindings(stage)?;
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
    }
    let declared_stage_nodes = declared_stage_nodes
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>();
    let declared_graph_nodes = declared_graph_nodes(suite);
    for stage in &suite.stages {
        let node_id = stage
            .stage_instance_id
            .as_deref()
            .unwrap_or(stage.stage.as_str());
        for upstream in &stage.upstream_stage_instance_ids {
            if !declared_stage_nodes.contains(upstream) {
                return Err(BenchError::InvalidPolicy(format!(
                    "suite stage {node_id} references unknown upstream stage node {upstream}"
                )));
            }
        }
    }
    let mut seen_edges = std::collections::BTreeSet::new();
    for edge in &suite.edges {
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
            edge.from.as_str(),
            edge.to.as_str(),
            edge.from_output_id.as_deref(),
            edge.to_input_id.as_deref(),
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
        validate_edge_ports(edge, &declared_graph_nodes)?;
    }
    validate_suite_analysis_requirements(suite)?;
    validate_suite_dag(suite)?;
    Ok(())
}
