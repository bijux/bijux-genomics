use std::collections::{BTreeMap, HashSet};

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use bijux_core::contract::PlanPolicy;
use bijux_core::contract::{ArtifactRef, ToolConstraints};
use bijux_core::prelude::ContainerImageRefV1;

use crate::stage_plan::{PlanDecisionReason, StagePlanV1};
use sha2::Digest;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlannerContractV1 {
    pub stage_id: String,
    pub tool_id: String,
    pub tool_version: Option<String>,
    pub image_ref: Option<ContainerImageRefV1>,
    pub parameters_json: serde_json::Value,
    pub effective_parameters_json: serde_json::Value,
    pub inputs: Vec<ArtifactRef>,
    pub outputs: Vec<ArtifactRef>,
    pub resources: ToolConstraints,
    pub reason: PlanDecisionReason,
}

impl From<&StagePlanV1> for PlannerContractV1 {
    fn from(stage: &StagePlanV1) -> Self {
        let tool_version = if stage.tool_version.trim().is_empty() {
            None
        } else {
            Some(stage.tool_version.clone())
        };
        let image_ref = if stage.image.image.trim().is_empty() {
            None
        } else {
            Some(stage.image.clone())
        };
        Self {
            stage_id: stage.stage_id.to_string(),
            tool_id: stage.tool_id.to_string(),
            tool_version,
            image_ref,
            parameters_json: stage.params.clone(),
            effective_parameters_json: stage.effective_params.clone(),
            inputs: stage.io.inputs.clone(),
            outputs: stage.io.outputs.clone(),
            resources: stage.resources.clone(),
            reason: stage.reason.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlanEdge {
    from: String,
    to: String,
}

impl PlanEdge {
    #[must_use]
    pub fn new(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
        }
    }

    #[must_use]
    pub fn from(&self) -> &str {
        &self.from
    }

    #[must_use]
    pub fn to(&self) -> &str {
        &self.to
    }
}

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
        stages.sort_by(|a, b| a.stage_id.0.cmp(&b.stage_id.0));
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
        Ok(bijux_core::contract::canonical::canonicalize_json_value(
            &value,
        ))
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
        if !id_catalog.insert(stage.stage_id.to_string()) {
            return Err(anyhow!("duplicate stage id in plan: {}", stage.stage_id.0));
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
        edges.push((edge.from.clone(), edge.to.clone()));
    }
    ensure_plan_is_dag(&id_catalog, &edges)?;
    Ok(())
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
    for window in stages.windows(2) {
        if let [from, to] = window {
            edges.push(PlanEdge::new(
                from.stage_id.to_string(),
                to.stage_id.to_string(),
            ));
        }
    }
    edges
}
