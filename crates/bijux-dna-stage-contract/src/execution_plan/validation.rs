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
    if plan.pipeline_id().trim().is_empty() {
        return Err(anyhow!("execution plan pipeline_id is empty"));
    }
    if plan.planner_version().trim().is_empty() {
        return Err(anyhow!("execution plan planner_version is empty"));
    }
    let mut id_catalog = HashSet::new();
    for stage in plan.stages() {
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
        ensure_unique_stage_artifacts(stage, "input", stage.io.inputs.iter().map(|artifact| {
            artifact.name.as_str()
        }))?;
        ensure_unique_stage_artifacts(stage, "output", stage.io.outputs.iter().map(|artifact| {
            artifact.name.as_str()
        }))?;
        if stage.resources.mem_gb == 0 || stage.resources.threads == 0 {
            return Err(anyhow!("stage {} missing resource hints", stage.stage_id.0));
        }
    }
    let stage_by_node_id = plan
        .stages()
        .iter()
        .map(|stage| (stage_node_id(stage), stage))
        .collect::<BTreeMap<_, _>>();
    let mut edges = Vec::new();
    for edge in plan.edges() {
        if edge.from().trim().is_empty() || edge.to().trim().is_empty() {
            return Err(anyhow!("plan edge has empty endpoint"));
        }
        if edge.from() == edge.to() {
            return Err(anyhow!("plan edge self-loop {}", edge.from()));
        }
        if !id_catalog.contains(edge.from()) || !id_catalog.contains(edge.to()) {
            return Err(anyhow!(
                "plan edge references unknown stage: {} -> {}",
                edge.from(),
                edge.to()
            ));
        }
        let from_stage = stage_by_node_id.get(edge.from()).ok_or_else(|| {
            anyhow!(
                "plan edge {} -> {} could not resolve source stage after validation",
                edge.from(),
                edge.to()
            )
        })?;
        let to_stage = stage_by_node_id.get(edge.to()).ok_or_else(|| {
            anyhow!(
                "plan edge {} -> {} could not resolve target stage after validation",
                edge.from(),
                edge.to()
            )
        })?;
        match (edge.from_output_id(), edge.to_input_id()) {
            (Some(from_output_id), Some(to_input_id)) => {
                if from_output_id.trim().is_empty() || to_input_id.trim().is_empty() {
                    return Err(anyhow!(
                        "plan edge {} -> {} has empty artifact binding",
                        edge.from(),
                        edge.to()
                    ));
                }
                if !from_stage
                    .io
                    .outputs
                    .iter()
                    .any(|artifact| artifact.name.as_str() == from_output_id)
                {
                    return Err(anyhow!(
                        "plan edge {} -> {} references unknown output artifact {}",
                        edge.from(),
                        edge.to(),
                        from_output_id
                    ));
                }
                if !stage_input_binding_exists(to_stage, to_input_id) {
                    return Err(anyhow!(
                        "plan edge {} -> {} references unknown input artifact {}",
                        edge.from(),
                        edge.to(),
                        to_input_id
                    ));
                }
            }
            (None, None) => {}
            _ => {
                return Err(anyhow!(
                    "plan edge {} -> {} must set both from_output_id and to_input_id together",
                    edge.from(),
                    edge.to()
                ));
            }
        }
        edges.push((edge.from().to_string(), edge.to().to_string()));
    }
    ensure_plan_is_dag(&id_catalog, &edges)?;
    Ok(())
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
    stage
        .stage_instance_id
        .as_ref()
        .map(ToString::to_string)
        .unwrap_or_else(|| stage.stage_id.to_string())
}
