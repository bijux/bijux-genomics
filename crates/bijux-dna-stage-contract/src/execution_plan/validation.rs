use std::collections::{BTreeMap, HashSet};

use anyhow::{anyhow, Result};

use bijux_dna_core::id_catalog;

use crate::{ExecutionPlan, StagePlanV1};

#[derive(Debug, Clone)]
pub struct PlanValidationContext<'a> {
    pub allowed_id_catalog: Option<&'a HashSet<String>>,
    pub allowed_tool_ids: Option<&'a HashSet<String>>,
}

/// # Errors
/// Returns an error if the plan fails validation checks.
pub fn lint_execution_plan(plan: &ExecutionPlan) -> Result<()> {
    validate_plan_metadata(plan)?;
    let id_catalog = validate_stages(plan)?;
    let stage_by_node_id =
        plan.stages().iter().map(|stage| (stage_node_id(stage), stage)).collect::<BTreeMap<_, _>>();
    let edges = validate_edges(plan, &id_catalog, &stage_by_node_id)?;
    ensure_plan_is_dag(&id_catalog, &edges)?;
    Ok(())
}

fn validate_plan_metadata(plan: &ExecutionPlan) -> Result<()> {
    if plan.pipeline_id().trim().is_empty() {
        return Err(anyhow!("execution plan pipeline_id is empty"));
    }
    if plan.planner_version().trim().is_empty() {
        return Err(anyhow!("execution plan planner_version is empty"));
    }
    Ok(())
}

fn validate_stages(plan: &ExecutionPlan) -> Result<HashSet<String>> {
    let mut id_catalog = HashSet::new();
    for stage in plan.stages() {
        validate_stage(stage, &mut id_catalog)?;
    }
    Ok(id_catalog)
}

fn validate_stage(stage: &StagePlanV1, id_catalog: &mut HashSet<String>) -> Result<()> {
    let node_id = stage_node_id(stage);
    if !id_catalog.insert(node_id.clone()) {
        return Err(anyhow!("duplicate stage node id in plan: {node_id}"));
    }
    if stage.io.inputs.is_empty() {
        return Err(anyhow!("stage {} missing declared inputs", stage.stage_id.0));
    }
    if stage.io.outputs.is_empty() {
        return Err(anyhow!("stage {} missing declared outputs", stage.stage_id.0));
    }
    ensure_unique_stage_artifacts(
        stage,
        "input",
        stage.io.inputs.iter().map(|artifact| artifact.name.as_str()),
    )?;
    ensure_unique_stage_artifacts(
        stage,
        "output",
        stage.io.outputs.iter().map(|artifact| artifact.name.as_str()),
    )?;
    if stage.resources.mem_gb == 0 || stage.resources.threads == 0 {
        return Err(anyhow!("stage {} missing resource hints", stage.stage_id.0));
    }
    Ok(())
}

fn validate_edges(
    plan: &ExecutionPlan,
    id_catalog: &HashSet<String>,
    stage_by_node_id: &BTreeMap<String, &StagePlanV1>,
) -> Result<Vec<(String, String)>> {
    let mut edges = Vec::new();
    for edge in plan.edges() {
        validate_edge_endpoints(edge.from(), edge.to(), id_catalog)?;
        let from_stage = resolve_edge_stage(stage_by_node_id, edge.from(), edge.to(), true)?;
        let to_stage = resolve_edge_stage(stage_by_node_id, edge.from(), edge.to(), false)?;
        validate_edge_bindings(
            edge.from(),
            edge.to(),
            edge.from_output_id(),
            edge.to_input_id(),
            from_stage,
            to_stage,
        )?;
        edges.push((edge.from().to_string(), edge.to().to_string()));
    }
    Ok(edges)
}

fn validate_edge_endpoints(from: &str, to: &str, id_catalog: &HashSet<String>) -> Result<()> {
    if from.trim().is_empty() || to.trim().is_empty() {
        return Err(anyhow!("plan edge has empty endpoint"));
    }
    if from == to {
        return Err(anyhow!("plan edge self-loop {from}"));
    }
    if !id_catalog.contains(from) || !id_catalog.contains(to) {
        return Err(anyhow!("plan edge references unknown stage: {from} -> {to}"));
    }
    Ok(())
}

fn resolve_edge_stage<'a>(
    stage_by_node_id: &'a BTreeMap<String, &'a StagePlanV1>,
    from: &str,
    to: &str,
    source: bool,
) -> Result<&'a StagePlanV1> {
    let node_id = if source { from } else { to };
    let role = if source { "source" } else { "target" };
    stage_by_node_id.get(node_id).copied().ok_or_else(|| {
        anyhow!("plan edge {from} -> {to} could not resolve {role} stage after validation")
    })
}

fn validate_edge_bindings(
    from: &str,
    to: &str,
    from_output_id: Option<&str>,
    to_input_id: Option<&str>,
    from_stage: &StagePlanV1,
    to_stage: &StagePlanV1,
) -> Result<()> {
    match (from_output_id, to_input_id) {
        (Some(from_output_id), Some(to_input_id)) => {
            if from_output_id.trim().is_empty() || to_input_id.trim().is_empty() {
                return Err(anyhow!("plan edge {from} -> {to} has empty artifact binding"));
            }
            if !from_stage
                .io
                .outputs
                .iter()
                .any(|artifact| artifact.name.as_str() == from_output_id)
            {
                return Err(anyhow!(
                    "plan edge {from} -> {to} references unknown output artifact {from_output_id}"
                ));
            }
            if !stage_input_binding_exists(to_stage, to_input_id) {
                return Err(anyhow!(
                    "plan edge {from} -> {to} references unknown input artifact {to_input_id}"
                ));
            }
            Ok(())
        }
        (None, None) => Ok(()),
        _ => Err(anyhow!(
            "plan edge {from} -> {to} must set both from_output_id and to_input_id together"
        )),
    }
}

fn ensure_unique_stage_artifacts<'a>(
    stage: &StagePlanV1,
    direction: &str,
    artifact_names: impl Iterator<Item = &'a str>,
) -> Result<()> {
    let mut seen = HashSet::new();
    for artifact_name in artifact_names {
        if !seen.insert(artifact_name) {
            return Err(anyhow!(
                "stage {} has duplicate {direction} artifact {artifact_name}",
                stage.stage_id.0
            ));
        }
    }
    Ok(())
}

fn stage_input_binding_exists(to_stage: &StagePlanV1, to_input_id: &str) -> bool {
    if to_stage.io.inputs.iter().any(|artifact| artifact.name.as_str() == to_input_id) {
        return true;
    }
    to_stage.stage_id.as_str() == id_catalog::FASTQ_QC_POST
        && to_input_id == "qc_artifacts"
        && !to_stage.io.inputs.is_empty()
}

fn ensure_plan_is_dag(id_catalog: &HashSet<String>, edges: &[(String, String)]) -> Result<()> {
    let mut adjacency: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for (from, to) in edges {
        adjacency.entry(from.as_str()).or_default().push(to.as_str());
    }
    let mut visiting = HashSet::new();
    let mut visited = HashSet::new();
    for stage in id_catalog {
        visit(stage, &adjacency, &mut visiting, &mut visited)?;
    }
    Ok(())
}

fn visit<'a>(
    node: &'a str,
    adjacency: &BTreeMap<&'a str, Vec<&'a str>>,
    visiting: &mut HashSet<&'a str>,
    visited: &mut HashSet<&'a str>,
) -> Result<()> {
    if visited.contains(node) {
        return Ok(());
    }
    if !visiting.insert(node) {
        return Err(anyhow!("cycle detected in execution plan at {node}"));
    }
    if let Some(children) = adjacency.get(node) {
        for &child in children {
            visit(child, adjacency, visiting, visited)?;
        }
    }
    visiting.remove(node);
    visited.insert(node);
    Ok(())
}

#[must_use]
pub(crate) fn stage_node_id(stage: &StagePlanV1) -> String {
    stage.stage_instance_id.as_ref().map_or_else(|| stage.stage_id.to_string(), ToString::to_string)
}
