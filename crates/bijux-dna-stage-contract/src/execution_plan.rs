use std::collections::{BTreeMap, HashSet};

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use bijux_dna_core::contract::PlanPolicy;
use bijux_dna_core::id_catalog;

use crate::{PlanEdge, StagePlanV1};
use sha2::Digest;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionPlan {
    schema_version: String,
    pipeline_id: String,
    planner_version: String,
    policy: PlanPolicy,
    stages: Vec<StagePlanV1>,
    edges: Vec<PlanEdge>,
}

#[derive(Debug, Clone)]
pub struct PlanValidationContext<'a> {
    pub allowed_id_catalog: Option<&'a HashSet<String>>,
    pub allowed_tool_ids: Option<&'a HashSet<String>>,
}

impl ExecutionPlan {
    #[must_use]
    pub fn schema_version(&self) -> &str {
        &self.schema_version
    }

    #[must_use]
    pub fn pipeline_id(&self) -> &str {
        &self.pipeline_id
    }

    #[must_use]
    pub fn planner_version(&self) -> &str {
        &self.planner_version
    }

    #[must_use]
    pub fn policy(&self) -> PlanPolicy {
        self.policy
    }

    #[must_use]
    pub fn stages(&self) -> &[StagePlanV1] {
        &self.stages
    }

    #[must_use]
    pub fn edges(&self) -> &[PlanEdge] {
        &self.edges
    }

    /// # Errors
    /// Returns an error if the plan fails validation checks.
    pub fn new(
        pipeline_id: impl Into<String>,
        planner_version: impl Into<String>,
        policy: PlanPolicy,
        stages: Vec<StagePlanV1>,
        edges: Vec<PlanEdge>,
    ) -> Result<Self> {
        let mut stages = stages;
        stages.sort_by_key(stage_node_id);
        let mut edges = edges;
        edges.sort_by(|a, b| match a.from.cmp(&b.from) {
            std::cmp::Ordering::Equal => a.to.cmp(&b.to),
            other => other,
        });
        let plan = Self {
            schema_version: "bijux.execution_plan.v1".to_string(),
            pipeline_id: pipeline_id.into(),
            planner_version: planner_version.into(),
            policy,
            stages,
            edges,
        };
        lint_execution_plan(&plan)?;
        Ok(plan)
    }

    /// # Errors
    /// Returns an error if the plan violates strict completeness requirements.
    pub fn validate_strict(&self, context: &PlanValidationContext<'_>) -> Result<()> {
        lint_execution_plan(self)?;
        let mut id_catalog = HashSet::new();
        for stage in &self.stages {
            id_catalog.insert(stage.stage_id.to_string());
            if stage.tool_id.0.trim().is_empty() {
                return Err(anyhow!("stage {} missing tool_id", stage.stage_id.0));
            }
            if stage.tool_version.trim().is_empty() && stage.image.image.trim().is_empty() {
                return Err(anyhow!(
                    "stage {} missing tool_version or image_ref",
                    stage.stage_id.0
                ));
            }
            if stage.params.is_null() {
                return Err(anyhow!(
                    "stage {} missing parameters_json",
                    stage.stage_id.0
                ));
            }
            if stage.effective_params.is_null() {
                return Err(anyhow!(
                    "stage {} missing effective_parameters_json",
                    stage.stage_id.0
                ));
            }
            if stage.io.inputs.is_empty() || stage.io.outputs.is_empty() {
                return Err(anyhow!(
                    "stage {} missing declared inputs/outputs",
                    stage.stage_id.0
                ));
            }
            if stage.resources.runtime.trim().is_empty()
                || stage.resources.mem_gb == 0
                || stage.resources.tmp_gb == 0
                || stage.resources.threads == 0
            {
                return Err(anyhow!(
                    "stage {} missing complete resources",
                    stage.stage_id.0
                ));
            }
            if stage.reason.summary.trim().is_empty() {
                return Err(anyhow!("stage {} missing reason", stage.stage_id.0));
            }
        }
        if let Some(allowed) = context.allowed_id_catalog {
            for stage_id in &id_catalog {
                if !allowed.contains(stage_id) {
                    return Err(anyhow!("unknown stage id in plan: {stage_id}"));
                }
            }
        }
        if let Some(allowed) = context.allowed_tool_ids {
            for stage in &self.stages {
                if !allowed.contains(stage.tool_id.as_str()) {
                    return Err(anyhow!("unknown tool id in plan: {}", stage.tool_id.0));
                }
            }
        }
        Ok(())
    }

    /// # Errors
    /// Returns an error if canonical JSON serialization fails.
    pub fn canonical_json(&self) -> Result<serde_json::Value> {
        let value = serde_json::to_value(self)?;
        Ok(bijux_dna_core::contract::canonical::canonicalize_json_value(&value))
    }

    /// # Errors
    /// Returns an error if canonical JSON serialization fails.
    pub fn plan_hash(&self) -> Result<String> {
        let canonical = self.canonical_json()?;
        let bytes = serde_json::to_vec(&canonical)?;
        let mut hasher = sha2::Sha256::new();
        hasher.update(bytes);
        Ok(format!("{:x}", hasher.finalize()))
    }
}

/// # Errors
/// Returns an error if the plan fails validation checks.
pub fn lint_execution_plan(plan: &ExecutionPlan) -> Result<()> {
    if plan.pipeline_id.trim().is_empty() {
        return Err(anyhow!("execution plan pipeline_id is empty"));
    }
    if plan.planner_version.trim().is_empty() {
        return Err(anyhow!("execution plan planner_version is empty"));
    }
    let mut id_catalog = HashSet::new();
    for stage in &plan.stages {
        let node_id = stage_node_id(stage);
        if !id_catalog.insert(node_id.clone()) {
            return Err(anyhow!("duplicate stage node id in plan: {node_id}"));
        }
        if stage.io.inputs.is_empty() {
            return Err(anyhow!(
                "stage {} missing declared inputs",
                stage.stage_id.0
            ));
        }
        if stage.io.outputs.is_empty() {
            return Err(anyhow!(
                "stage {} missing declared outputs",
                stage.stage_id.0
            ));
        }
        if stage.resources.mem_gb == 0 || stage.resources.threads == 0 {
            return Err(anyhow!("stage {} missing resource hints", stage.stage_id.0));
        }
    }
    let mut edges = Vec::new();
    for edge in &plan.edges {
        if edge.from == edge.to {
            return Err(anyhow!("plan edge self-loop {}", edge.from));
        }
        if !id_catalog.contains(&edge.from) || !id_catalog.contains(&edge.to) {
            return Err(anyhow!(
                "plan edge references unknown stage: {} -> {}",
                edge.from,
                edge.to
            ));
        }
        let from_stage = plan
            .stages
            .iter()
            .find(|stage| stage_node_id(stage) == edge.from)
            .ok_or_else(|| {
                anyhow!(
                    "plan edge {} -> {} could not resolve source stage after validation",
                    edge.from,
                    edge.to
                )
            })?;
        let to_stage = plan
            .stages
            .iter()
            .find(|stage| stage_node_id(stage) == edge.to)
            .ok_or_else(|| {
                anyhow!(
                    "plan edge {} -> {} could not resolve target stage after validation",
                    edge.from,
                    edge.to
                )
            })?;
        match (edge.from_output_id(), edge.to_input_id()) {
            (Some(from_output_id), Some(to_input_id)) => {
                if !from_stage
                    .io
                    .outputs
                    .iter()
                    .any(|artifact| artifact.name.as_str() == from_output_id)
                {
                    return Err(anyhow!(
                        "plan edge {} -> {} references unknown output artifact {}",
                        edge.from,
                        edge.to,
                        from_output_id
                    ));
                }
                if !stage_input_binding_exists(to_stage, to_input_id) {
                    return Err(anyhow!(
                        "plan edge {} -> {} references unknown input artifact {}",
                        edge.from,
                        edge.to,
                        to_input_id
                    ));
                }
            }
            (None, None) => {}
            _ => {
                return Err(anyhow!(
                    "plan edge {} -> {} must set both from_output_id and to_input_id together",
                    edge.from,
                    edge.to
                ));
            }
        }
        edges.push((edge.from.clone(), edge.to.clone()));
    }
    ensure_plan_is_dag(&id_catalog, &edges)?;
    Ok(())
}

fn stage_input_binding_exists(to_stage: &StagePlanV1, to_input_id: &str) -> bool {
    if to_stage
        .io
        .inputs
        .iter()
        .any(|artifact| artifact.name.as_str() == to_input_id)
    {
        return true;
    }
    to_stage.stage_id.as_str() == id_catalog::FASTQ_QC_POST
        && to_input_id == "qc_artifacts"
        && !to_stage.io.inputs.is_empty()
}

fn ensure_plan_is_dag(id_catalog: &HashSet<String>, edges: &[(String, String)]) -> Result<()> {
    let mut adjacency: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for (from, to) in edges {
        adjacency
            .entry(from.as_str())
            .or_default()
            .push(to.as_str());
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
pub fn default_edges_for_stages(stages: &[StagePlanV1]) -> Vec<PlanEdge> {
    let mut edges = Vec::new();
    for (to_idx, to_stage) in stages.iter().enumerate() {
        let mut artifact_edges = Vec::new();
        for input in &to_stage.io.inputs {
            let Some((from_stage, output)) = stages[..to_idx].iter().rev().find_map(|candidate| {
                candidate
                    .io
                    .outputs
                    .iter()
                    .find(|output| output.name == input.name)
                    .map(|output| (candidate, output))
            }) else {
                continue;
            };
            artifact_edges.push(PlanEdge::with_artifact_binding(
                stage_node_id(from_stage),
                stage_node_id(to_stage),
                output.name.as_str(),
                input.name.as_str(),
            ));
        }
        if artifact_edges.is_empty() {
            if let Some(from_stage) = to_idx.checked_sub(1).and_then(|idx| stages.get(idx)) {
                edges.push(PlanEdge::new(
                    stage_node_id(from_stage),
                    stage_node_id(to_stage),
                ));
            }
        } else {
            edges.extend(artifact_edges);
        }
    }
    edges.sort_by(|left, right| {
        left.from
            .cmp(&right.from)
            .then_with(|| left.to.cmp(&right.to))
            .then_with(|| left.from_output_id.cmp(&right.from_output_id))
            .then_with(|| left.to_input_id.cmp(&right.to_input_id))
    });
    edges.dedup_by(|left, right| {
        left.from == right.from
            && left.to == right.to
            && left.from_output_id == right.from_output_id
            && left.to_input_id == right.to_input_id
    });
    edges
}

fn stage_node_id(stage: &StagePlanV1) -> String {
    stage
        .stage_instance_id
        .as_ref()
        .map(ToString::to_string)
        .unwrap_or_else(|| stage.stage_id.to_string())
}
