use std::collections::{BTreeMap, HashSet};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use std::fmt::Write as _;
use sha2::Digest;

use crate::contract::{ContractVersion, StageIO, ToolConstraints};
use crate::foundation::{BijuxError, CommandSpecV1, ContainerImageRefV1, Result};
use crate::id_catalog;
use crate::ids::{ArtifactId, PipelineId, StageId, StepId};

use super::policy::{PlanPolicy, RetryPolicy};

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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    from_output_id: Option<ArtifactId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    to_input_id: Option<ArtifactId>,
}

impl ExecutionEdge {
    #[must_use]
    pub fn new(from: StepId, to: StepId) -> Self {
        Self {
            from,
            to,
            from_output_id: None,
            to_input_id: None,
        }
    }

    #[must_use]
    pub fn with_artifact_binding(
        from: StepId,
        to: StepId,
        from_output_id: ArtifactId,
        to_input_id: ArtifactId,
    ) -> Self {
        Self {
            from,
            to,
            from_output_id: Some(from_output_id),
            to_input_id: Some(to_input_id),
        }
    }

    #[must_use]
    pub fn from(&self) -> &StepId {
        &self.from
    }

    #[must_use]
    pub fn to(&self) -> &StepId {
        &self.to
    }

    #[must_use]
    pub fn from_output_id(&self) -> Option<&ArtifactId> {
        self.from_output_id.as_ref()
    }

    #[must_use]
    pub fn to_input_id(&self) -> Option<&ArtifactId> {
        self.to_input_id.as_ref()
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
    #[serde(default)]
    deterministic_scheduler: bool,
    #[serde(default)]
    retry_policy: RetryPolicy,
    #[serde(default)]
    step_timeout_s: Option<u64>,
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
    pub fn deterministic_scheduler(&self) -> bool {
        self.deterministic_scheduler
    }

    #[must_use]
    pub fn retry_policy(&self) -> &RetryPolicy {
        &self.retry_policy
    }

    #[must_use]
    pub fn step_timeout_s(&self) -> Option<u64> {
        self.step_timeout_s
    }

    #[must_use]
    pub fn steps(&self) -> &[ExecutionStep] {
        &self.steps
    }

    #[must_use]
    pub fn edges(&self) -> &[ExecutionEdge] {
        &self.edges
    }

    #[must_use]
    pub fn step_by_id(&self, step_id: &str) -> Option<&ExecutionStep> {
        self.steps
            .iter()
            .find(|step| step.step_id.as_str() == step_id)
    }

    /// # Errors
    /// Returns an error if the graph is cyclic or references unknown steps.
    pub fn topological_step_ids(&self) -> Result<Vec<&StepId>> {
        validate_acyclic(self)?;
        let mut incoming: BTreeMap<&str, usize> = BTreeMap::new();
        let mut outgoing: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
        for step in &self.steps {
            incoming.insert(step.step_id.as_str(), 0);
        }
        for edge in &self.edges {
            *incoming.entry(edge.to().as_str()).or_insert(0) += 1;
            outgoing
                .entry(edge.from().as_str())
                .or_default()
                .push(edge.to().as_str());
        }
        let mut ready = incoming
            .iter()
            .filter_map(|(id, count)| if *count == 0 { Some(*id) } else { None })
            .collect::<Vec<_>>();
        ready.sort_unstable();
        ready.reverse();
        let mut order = Vec::with_capacity(self.steps.len());
        while let Some(node_id) = ready.pop() {
            let step = self.step_by_id(node_id).ok_or_else(|| {
                BijuxError::validation(format!(
                    "execution graph topological walk could not resolve step {node_id}"
                ))
            })?;
            order.push(&step.step_id);
            if let Some(children) = outgoing.get(node_id) {
                let mut released = Vec::new();
                for child in children {
                    if let Some(count) = incoming.get_mut(child) {
                        *count -= 1;
                        if *count == 0 {
                            released.push(*child);
                        }
                    }
                }
                released.sort_unstable();
                for child in released.into_iter().rev() {
                    ready.push(child);
                }
            }
        }
        Ok(order)
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
            pipeline_id,
            planner_version: planner_version.into(),
            policy,
            deterministic_scheduler: true,
            retry_policy: RetryPolicy::default(),
            step_timeout_s: None,
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
        let bytes = crate::contract::canonical::to_canonical_json_bytes(self)?;
        let mut hasher = sha2::Sha256::new();
        hasher.update(bytes);
        let digest = hasher.finalize();
        let mut hex = String::with_capacity(digest.len() * 2);
        for byte in digest {
            let _ = write!(&mut hex, "{byte:02x}");
        }
        Ok(hex)
    }

    /// # Errors
    /// Returns an error if the normalized graph fails validation.
    pub fn normalize(&self) -> Result<Self> {
        let mut graph = self.clone();
        graph.steps.sort_by(|a, b| a.step_id.0.cmp(&b.step_id.0));
        graph.edges.sort_by(|a, b| match a.from.0.cmp(&b.from.0) {
            std::cmp::Ordering::Equal => a.to.0.cmp(&b.to.0),
            other => other,
        });
        graph.validate()?;
        Ok(graph)
    }

    #[must_use]
    pub fn with_retry_policy(mut self, retry_policy: RetryPolicy) -> Self {
        self.retry_policy = retry_policy;
        self
    }

    #[must_use]
    pub fn with_deterministic_scheduler(mut self, deterministic: bool) -> Self {
        self.deterministic_scheduler = deterministic;
        self
    }

    #[must_use]
    pub fn with_step_timeout(mut self, timeout_s: Option<u64>) -> Self {
        self.step_timeout_s = timeout_s;
        self
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
    validate_graph_identity(graph)?;
    let by_id = validate_graph_steps(graph)?;
    validate_graph_edges(graph, &by_id)
}

fn validate_graph_identity(graph: &ExecutionGraph) -> Result<()> {
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
    Ok(())
}

fn validate_graph_steps(graph: &ExecutionGraph) -> Result<BTreeMap<&str, &ExecutionStep>> {
    let mut step_ids = HashSet::new();
    let mut by_id: BTreeMap<&str, &ExecutionStep> = BTreeMap::new();
    for step in &graph.steps {
        validate_graph_step(step, &mut step_ids)?;
        by_id.insert(step.step_id.as_str(), step);
    }
    Ok(by_id)
}

fn validate_graph_step(step: &ExecutionStep, step_ids: &mut HashSet<String>) -> Result<()> {
    if !step_ids.insert(step.step_id.to_string()) {
        return Err(BijuxError::validation(format!(
            "duplicate step id {}",
            step.step_id.0
        )));
    }
    if step.stage_id.as_str().trim().is_empty() {
        return Err(BijuxError::validation(format!(
            "step {} missing stage_id",
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
    let mut artifacts = HashSet::new();
    for artifact in step.io.inputs.iter().chain(step.io.outputs.iter()) {
        if artifact.name.as_str().trim().is_empty() {
            return Err(BijuxError::validation(format!(
                "step {} has artifact with empty name",
                step.step_id.0
            )));
        }
        if artifact.path.as_os_str().is_empty() {
            return Err(BijuxError::validation(format!(
                "step {} has artifact with empty path",
                step.step_id.0
            )));
        }
        if !artifacts.insert(artifact.name.to_string()) {
            return Err(BijuxError::validation(format!(
                "step {} has duplicate artifact {}",
                step.step_id.0,
                artifact.name.as_str()
            )));
        }
    }
    Ok(())
}

fn validate_graph_edges(
    graph: &ExecutionGraph,
    by_id: &BTreeMap<&str, &ExecutionStep>,
) -> Result<()> {
    for edge in &graph.edges {
        if !by_id.contains_key(edge.from().as_str()) || !by_id.contains_key(edge.to().as_str()) {
            return Err(BijuxError::validation(format!(
                "edge references unknown step: {} -> {}",
                edge.from().0,
                edge.to().0
            )));
        }
        let from_step = by_id.get(edge.from().as_str()).copied().ok_or_else(|| {
            BijuxError::validation(format!(
                "edge {} -> {} could not resolve source step after validation",
                edge.from().0,
                edge.to().0
            ))
        })?;
        let to_step = by_id.get(edge.to().as_str()).copied().ok_or_else(|| {
            BijuxError::validation(format!(
                "edge {} -> {} could not resolve target step after validation",
                edge.from().0,
                edge.to().0
            ))
        })?;
        match (edge.from_output_id(), edge.to_input_id()) {
            (Some(from_output_id), Some(to_input_id)) => {
                if !from_step
                    .io
                    .outputs
                    .iter()
                    .any(|artifact| artifact.name == *from_output_id)
                {
                    return Err(BijuxError::validation(format!(
                        "edge {} -> {} references unknown output artifact {}",
                        edge.from().as_str(),
                        edge.to().as_str(),
                        from_output_id.as_str()
                    )));
                }
                if !step_input_binding_exists(to_step, to_input_id.as_str()) {
                    return Err(BijuxError::validation(format!(
                        "edge {} -> {} references unknown input artifact {}",
                        edge.from().as_str(),
                        edge.to().as_str(),
                        to_input_id.as_str()
                    )));
                }
            }
            (None, None) => {}
            _ => {
                return Err(BijuxError::validation(format!(
                    "edge {} -> {} must set both from_output_id and to_input_id together",
                    edge.from().as_str(),
                    edge.to().as_str()
                )));
            }
        }
    }
    Ok(())
}

fn step_input_binding_exists(step: &ExecutionStep, input_id: &str) -> bool {
    if step
        .io
        .inputs
        .iter()
        .any(|artifact| artifact.name.as_str() == input_id)
    {
        return true;
    }
    step.stage_id.as_str() == id_catalog::FASTQ_QC_POST
        && input_id == "qc_artifacts"
        && !step.io.inputs.is_empty()
}

fn validate_acyclic(graph: &ExecutionGraph) -> Result<()> {
    let mut incoming: BTreeMap<&str, usize> = BTreeMap::new();
    let mut outgoing: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
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
        return Err(BijuxError::validation("execution graph contains a cycle"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{ExecutionEdge, ExecutionGraph, ExecutionStep};
    use crate::contract::{ArtifactRef, ArtifactRole, PlanPolicy, StageIO, ToolConstraints};
    use crate::foundation::{CommandSpecV1, ContainerImageRefV1};
    use crate::id_catalog;
    use crate::ids::{ArtifactId, StageId, StepId};
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    fn step(step_id: &str, stage_id: &str) -> ExecutionStep {
        ExecutionStep {
            step_id: StepId::new(step_id.to_string()),
            stage_id: StageId::new(stage_id.to_string()),
            command: CommandSpecV1 {
                template: vec!["tool".to_string()],
            },
            image: ContainerImageRefV1 {
                image: "img".to_string(),
                digest: Some("sha256:test".to_string()),
            },
            resources: ToolConstraints::default(),
            io: StageIO {
                inputs: vec![ArtifactRef::required(
                    ArtifactId::new("in".to_string()),
                    PathBuf::from("in.fastq.gz"),
                    ArtifactRole::Reads,
                )],
                outputs: vec![ArtifactRef::required(
                    ArtifactId::new("out".to_string()),
                    PathBuf::from("out.fastq.gz"),
                    ArtifactRole::TrimmedReads,
                )],
            },
            out_dir: PathBuf::from("out"),
            aux_images: BTreeMap::default(),
            expected_artifact_ids: vec![],
            metrics_schema_ids: vec![],
        }
    }

    #[test]
    fn execution_graph_exposes_step_lookup() {
        let graph = ExecutionGraph::new(
            "fastq-to-fastq__graph__v1",
            "planner",
            PlanPolicy::PreferAccuracy,
            vec![step("a", id_catalog::FASTQ_VALIDATE_READS)],
            vec![],
        )
        .unwrap_or_else(|err| panic!("graph should be valid: {err}"));
        assert_eq!(
            graph
                .step_by_id("a")
                .unwrap_or_else(|| panic!("expected step a"))
                .stage_id,
            StageId::from_static(id_catalog::FASTQ_VALIDATE_READS)
        );
        assert!(graph.step_by_id("missing").is_none());
    }

    #[test]
    fn execution_graph_topological_order_is_stable() {
        let graph = ExecutionGraph::new(
            "fastq-to-fastq__graph__v1",
            "planner",
            PlanPolicy::PreferAccuracy,
            vec![
                step("trim", id_catalog::FASTQ_TRIM),
                step("validate", id_catalog::FASTQ_VALIDATE_READS),
                step("report", id_catalog::FASTQ_QC_POST),
            ],
            vec![
                ExecutionEdge::new(
                    StepId::new("validate".to_string()),
                    StepId::new("trim".to_string()),
                ),
                ExecutionEdge::new(
                    StepId::new("trim".to_string()),
                    StepId::new("report".to_string()),
                ),
            ],
        )
        .unwrap_or_else(|err| panic!("graph should be valid: {err}"));
        let ordered = graph
            .topological_step_ids()
            .unwrap_or_else(|err| panic!("topological order should succeed: {err}"))
            .into_iter()
            .map(|step_id| step_id.as_str().to_string())
            .collect::<Vec<_>>();
        assert_eq!(ordered, vec!["validate", "trim", "report"]);
    }

    #[test]
    fn malformed_execution_graph_reports_unknown_edges_without_panicking() {
        let graph = ExecutionGraph::new(
            "fastq-to-fastq__graph__v1",
            "planner",
            PlanPolicy::PreferAccuracy,
            vec![step("trim", id_catalog::FASTQ_TRIM)],
            vec![],
        )
        .unwrap_or_else(|err| panic!("graph should be valid: {err}"));
        let mut encoded =
            serde_json::to_value(&graph).unwrap_or_else(|err| panic!("serialize graph: {err}"));
        encoded["edges"] = serde_json::json!([{
            "from": "trim",
            "to": "report"
        }]);
        let malformed: ExecutionGraph = serde_json::from_value(encoded)
            .unwrap_or_else(|err| panic!("deserialize malformed graph: {err}"));
        let error = match malformed.validate_strict() {
            Ok(()) => panic!("unknown edges must fail validation"),
            Err(error) => error,
        };
        assert!(error
            .to_string()
            .contains("edge references unknown step: trim -> report"));
    }
}
