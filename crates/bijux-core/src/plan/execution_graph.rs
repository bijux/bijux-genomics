use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::PathBuf;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use sha2::Digest;

use crate::contract::ToolConstraints;
use crate::ids::StageId;
use crate::plan::stage_plan::{StageIO, StagePlanV1};
use crate::plan::PlanPolicy;
use crate::primitives::{CommandSpecV1, ContainerImageRefV1};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionStep {
    pub step_id: StageId,
    pub command: CommandSpecV1,
    pub image: ContainerImageRefV1,
    pub resources: ToolConstraints,
    pub io: StageIO,
    pub out_dir: PathBuf,
    #[serde(default)]
    pub aux_images: BTreeMap<String, ContainerImageRefV1>,
    #[serde(default)]
    pub expected_artifact_ids: Vec<String>,
    #[serde(default)]
    pub metrics_schema_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionEdge {
    from: StageId,
    to: StageId,
}

impl ExecutionEdge {
    #[must_use]
    pub fn new(from: StageId, to: StageId) -> Self {
        Self { from, to }
    }

    #[must_use]
    pub fn from(&self) -> &StageId {
        &self.from
    }

    #[must_use]
    pub fn to(&self) -> &StageId {
        &self.to
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionGraph {
    schema_version: String,
    pipeline_id: String,
    planner_version: String,
    policy: PlanPolicy,
    steps: Vec<ExecutionStep>,
    edges: Vec<ExecutionEdge>,
}

impl From<&StagePlanV1> for ExecutionStep {
    fn from(plan: &StagePlanV1) -> Self {
        Self {
            step_id: plan.stage_id.clone(),
            command: plan.command.clone(),
            image: plan.image.clone(),
            resources: plan.resources.clone(),
            io: plan.io.clone(),
            out_dir: plan.out_dir.clone(),
            aux_images: plan.aux_images.clone(),
            expected_artifact_ids: Vec::new(),
            metrics_schema_ids: Vec::new(),
        }
    }
}

impl ExecutionGraph {
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
    pub fn steps(&self) -> &[ExecutionStep] {
        &self.steps
    }

    #[must_use]
    pub fn edges(&self) -> &[ExecutionEdge] {
        &self.edges
    }

    /// # Errors
    /// Returns an error if the graph fails validation checks.
    pub fn new(
        pipeline_id: impl Into<String>,
        planner_version: impl Into<String>,
        policy: PlanPolicy,
        steps: Vec<ExecutionStep>,
        edges: Vec<ExecutionEdge>,
    ) -> Result<Self> {
        let mut steps = steps;
        steps.sort_by(|a, b| a.step_id.0.cmp(&b.step_id.0));
        let mut edges = edges;
        edges.sort_by(|a, b| match a.from.0.cmp(&b.from.0) {
            std::cmp::Ordering::Equal => a.to.0.cmp(&b.to.0),
            other => other,
        });
        let graph = Self {
            schema_version: "bijux.execution_graph.v1".to_string(),
            pipeline_id: pipeline_id.into(),
            planner_version: planner_version.into(),
            policy,
            steps,
            edges,
        };
        lint_execution_graph(&graph)?;
        Ok(graph)
    }

    /// # Errors
    /// Returns an error if graph references unknown steps or missing IO.
    pub fn validate_strict(&self) -> Result<()> {
        lint_execution_graph(self)
    }

    /// # Errors
    /// Returns an error if canonical JSON serialization fails.
    pub fn canonical_json(&self) -> Result<serde_json::Value> {
        let value = serde_json::to_value(self)?;
        Ok(crate::primitives::hashing::canonicalize_json_value(&value))
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
/// Returns an error if the graph fails structure validation.
pub fn lint_execution_graph(graph: &ExecutionGraph) -> Result<()> {
    if graph.pipeline_id.trim().is_empty() {
        return Err(anyhow!("execution graph pipeline_id is empty"));
    }
    if graph.planner_version.trim().is_empty() {
        return Err(anyhow!("execution graph planner_version is empty"));
    }
    let mut step_ids = HashSet::new();
    for step in &graph.steps {
        if !step_ids.insert(step.step_id.to_string()) {
            return Err(anyhow!("duplicate step id {}", step.step_id.0));
        }
        if step.command.template.is_empty() {
            return Err(anyhow!("step {} missing command", step.step_id.0));
        }
        if step.image.image.trim().is_empty() {
            return Err(anyhow!("step {} missing image", step.step_id.0));
        }
        if step.io.inputs.is_empty() || step.io.outputs.is_empty() {
            return Err(anyhow!("step {} missing IO", step.step_id.0));
        }
    }
    let mut by_id: HashMap<&str, &ExecutionStep> = HashMap::new();
    for step in &graph.steps {
        by_id.insert(step.step_id.as_str(), step);
    }
    for edge in &graph.edges {
        if !by_id.contains_key(edge.from().as_str()) || !by_id.contains_key(edge.to().as_str()) {
            return Err(anyhow!(
                "edge references unknown step: {} -> {}",
                edge.from().0,
                edge.to().0
            ));
        }
    }
    Ok(())
}
