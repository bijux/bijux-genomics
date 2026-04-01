use super::DeclaredStageNodes;
use crate::contract::suite::{
    ensure_supported_stage, planner_owned_graph_stage, validate_stage_param_bindings,
    validate_stage_tools,
};
use crate::diagnostics::BenchError;
use crate::model::{BenchmarkStageSpec, BenchmarkSuiteSpec};

pub(super) fn validate_schema_version(
    suite: &BenchmarkSuiteSpec,
    schema_version: &str,
) -> Result<(), BenchError> {
    if suite.schema_version == schema_version {
        return Ok(());
    }
    Err(BenchError::InvalidPolicy(format!(
        "suite schema mismatch: {}",
        suite.schema_version
    )))
}

pub(super) fn validate_stage_definitions(
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

fn stage_node_id(stage: &BenchmarkStageSpec) -> &str {
    stage
        .stage_instance_id
        .as_deref()
        .unwrap_or(stage.stage.as_str())
}
