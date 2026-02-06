use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use sha2::Digest;

use crate::contract::{ContractVersion, StageIO, ToolConstraints};
use crate::execution::PlanPolicy;
use crate::ids::{ArtifactId, PipelineId, StageId, StepId};
use crate::primitives::{BijuxError, CommandSpecV1, ContainerImageRefV1, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionStep {
    pub step_id: StepId,
    pub stage_id: StageId,
    pub command: CommandSpecV1,
    pub image: ContainerImageRefV1,
    pub resources: ToolConstraints,
    pub io: StageIO,
    pub out_dir: PathBuf,
    #[serde(default)]
    pub aux_images: BTreeMap<String, ContainerImageRefV1>,
    #[serde(default)]
    pub expected_artifact_ids: Vec<ArtifactId>,
    #[serde(default)]
    pub metrics_schema_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionEdge {
    from: StepId,
    to: StepId,
}

impl ExecutionEdge {
    #[must_use]
    pub fn new(from: StepId, to: StepId) -> Self {
        Self { from, to }
    }

    #[must_use]
    pub fn from(&self) -> &StepId {
        &self.from
    }

    #[must_use]
    pub fn to(&self) -> &StepId {
        &self.to
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionGraph {
    schema_version: String,
    contract_version: ContractVersion,
    pipeline_id: PipelineId,
    planner_version: String,
    policy: PlanPolicy,
    steps: Vec<ExecutionStep>,
    edges: Vec<ExecutionEdge>,
}

impl ExecutionGraph {
    #[must_use]
    pub fn schema_version(&self) -> &str {
        &self.schema_version
    }

    #[must_use]
    pub fn pipeline_id(&self) -> &PipelineId {
        &self.pipeline_id
    }

    #[must_use]
    pub fn contract_version(&self) -> ContractVersion {
        self.contract_version
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
        let pipeline_id_raw = pipeline_id.into();
        let pipeline_id = PipelineId::try_from(pipeline_id_raw.as_str())?;
        let graph = Self {
            schema_version: "bijux.execution_graph.v1".to_string(),
            contract_version: ContractVersion::v1(),
            pipeline_id: pipeline_id.into(),
            planner_version: planner_version.into(),
            policy,
            steps,
            edges,
        };
        graph.validate()?;
        Ok(graph)
    }

    /// # Errors
    /// Returns an error if graph references unknown steps or missing IO.
    pub fn validate_strict(&self) -> Result<()> {
        self.validate()
    }

    /// # Errors
    /// Returns an error if canonical JSON serialization fails.
    pub fn hash(&self) -> Result<String> {
        let bytes = crate::primitives::hashing::to_canonical_json_bytes(self)?;
        let mut hasher = sha2::Sha256::new();
        hasher.update(bytes);
        Ok(format!("{:x}", hasher.finalize()))
    }

    /// # Errors
    /// Returns an error if the normalized graph fails validation.
    pub fn normalize(&self) -> Result<Self> {
        ExecutionGraph::new(
            self.pipeline_id.as_str(),
            self.planner_version.clone(),
            self.policy,
            self.steps.clone(),
            self.edges.clone(),
        )
    }

    /// # Errors
    /// Returns an error if validation fails.
    pub fn validate(&self) -> Result<()> {
        lint_execution_graph(self)?;
        validate_acyclic(self)
    }
}

/// # Errors
/// Returns an error if the graph fails structure validation.
pub fn lint_execution_graph(graph: &ExecutionGraph) -> Result<()> {
    if graph.pipeline_id.as_str().trim().is_empty() {
        return Err(BijuxError::validation(
            "execution graph pipeline_id is empty",
        ));
    }
    if graph.planner_version.trim().is_empty() {
        return Err(BijuxError::validation(
            "execution graph planner_version is empty",
        ));
    }
    let mut step_ids = HashSet::new();
    for step in &graph.steps {
        if !step_ids.insert(step.step_id.to_string()) {
            return Err(BijuxError::validation(format!(
                "duplicate step id {}",
                step.step_id.0
            )));
        }
        if step.command.template.is_empty() {
            return Err(BijuxError::validation(format!(
                "step {} missing command",
                step.step_id.0
            )));
        }
        if step.image.image.trim().is_empty() {
            return Err(BijuxError::validation(format!(
                "step {} missing image",
                step.step_id.0
            )));
        }
        if step.io.inputs.is_empty() || step.io.outputs.is_empty() {
            return Err(BijuxError::validation(format!(
                "step {} missing IO",
                step.step_id.0
            )));
        }
    }
    let mut by_id: HashMap<&str, &ExecutionStep> = HashMap::new();
    for step in &graph.steps {
        by_id.insert(step.step_id.as_str(), step);
    }
    for edge in &graph.edges {
        if !by_id.contains_key(edge.from().as_str()) || !by_id.contains_key(edge.to().as_str()) {
            return Err(BijuxError::validation(format!(
                "edge references unknown step: {} -> {}",
                edge.from().0,
                edge.to().0
            )));
        }
    }
    Ok(())
}

fn validate_acyclic(graph: &ExecutionGraph) -> Result<()> {
    let mut incoming: HashMap<&str, usize> = HashMap::new();
    let mut outgoing: HashMap<&str, Vec<&str>> = HashMap::new();
    for step in &graph.steps {
        incoming.insert(step.step_id.as_str(), 0);
    }
    for edge in &graph.edges {
        let from = edge.from().as_str();
        let to = edge.to().as_str();
        *incoming.entry(to).or_insert(0) += 1;
        outgoing.entry(from).or_default().push(to);
    }
    let mut queue: Vec<&str> = incoming
        .iter()
        .filter_map(|(id, count)| if *count == 0 { Some(*id) } else { None })
        .collect();
    let mut visited = 0usize;
    while let Some(node) = queue.pop() {
        visited += 1;
        if let Some(children) = outgoing.get(node) {
            for child in children {
                if let Some(count) = incoming.get_mut(child) {
                    *count -= 1;
                    if *count == 0 {
                        queue.push(child);
                    }
                }
            }
        }
    }
    if visited != graph.steps.len() {
        return Err(BijuxError::validation(
            "execution graph contains a cycle",
        ));
    }
    Ok(())
}
