use std::collections::{BTreeSet, HashMap, HashSet};

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::StagePlanV1;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanPolicy {
    PreferAccuracy,
    PreferSpeed,
    PreferMemory,
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

    pub fn new(
        pipeline_id: impl Into<String>,
        planner_version: impl Into<String>,
        policy: PlanPolicy,
        stages: Vec<StagePlanV1>,
        edges: Vec<PlanEdge>,
    ) -> Result<Self> {
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
}

pub fn lint_execution_plan(plan: &ExecutionPlan) -> Result<()> {
    if plan.pipeline_id.trim().is_empty() {
        return Err(anyhow!("execution plan pipeline_id is empty"));
    }
    if plan.planner_version.trim().is_empty() {
        return Err(anyhow!("execution plan planner_version is empty"));
    }
    let mut stage_ids = HashSet::new();
    for stage in &plan.stages {
        if !stage_ids.insert(stage.stage_id.0.clone()) {
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
            return Err(anyhow!(
                "stage {} missing resource hints",
                stage.stage_id.0
            ));
        }
    }
    let mut edges = Vec::new();
    for edge in &plan.edges {
        if edge.from == edge.to {
            return Err(anyhow!("plan edge self-loop {}", edge.from));
        }
        if !stage_ids.contains(&edge.from) || !stage_ids.contains(&edge.to) {
            return Err(anyhow!(
                "plan edge references unknown stage: {} -> {}",
                edge.from,
                edge.to
            ));
        }
        edges.push((edge.from.clone(), edge.to.clone()));
    }
    ensure_plan_is_dag(&stage_ids, &edges)?;
    Ok(())
}

fn ensure_plan_is_dag(stage_ids: &HashSet<String>, edges: &[(String, String)]) -> Result<()> {
    let mut adjacency: HashMap<&str, Vec<&str>> = HashMap::new();
    for (from, to) in edges {
        adjacency
            .entry(from.as_str())
            .or_default()
            .push(to.as_str());
    }
    let mut visiting = HashSet::new();
    let mut visited = HashSet::new();
    for stage in stage_ids {
        visit(stage, &adjacency, &mut visiting, &mut visited)?;
    }
    Ok(())
}

fn visit<'a>(
    node: &'a str,
    adjacency: &HashMap<&'a str, Vec<&'a str>>,
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

pub fn default_edges_for_stages(stages: &[StagePlanV1]) -> Vec<PlanEdge> {
    let mut edges = Vec::new();
    for window in stages.windows(2) {
        if let [from, to] = window {
            edges.push(PlanEdge::new(from.stage_id.0.clone(), to.stage_id.0.clone()));
        }
    }
    edges
}
