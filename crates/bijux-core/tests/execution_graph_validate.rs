use bijux_core::contract::execution::graph::{ExecutionEdge, ExecutionGraph, ExecutionStep};
use bijux_core::contract::{PlanPolicy, StageIO, ToolConstraints};
use bijux_core::foundation::{CommandSpecV1, ContainerImageRefV1};
use bijux_core::ids::{ArtifactId, PipelineId, StageId, StepId, ToolId};
use std::path::PathBuf;

fn step(step_id: &str) -> ExecutionStep {
    ExecutionStep {
        step_id: StepId::new(step_id),
        stage_id: StageId::new("fastq.trim"),
        command: CommandSpecV1 {
            tool_id: ToolId::new("fastp"),
            template: "fastp".to_string(),
            args: vec![],
            working_dir: None,
        },
        image: ContainerImageRefV1 {
            image: "fastp:latest".to_string(),
            digest: None,
        },
        resources: ToolConstraints::default(),
        io: StageIO {
            inputs: vec![ArtifactId::new("in")],
            outputs: vec![ArtifactId::new("out")],
            optional_outputs: Vec::new(),
        },
        out_dir: PathBuf::from("/tmp"),
        aux_images: Default::default(),
        expected_artifact_ids: Vec::new(),
        metrics_schema_ids: Vec::new(),
    }
}

#[test]
fn validate_rejects_cycles() {
    let a = step("a");
    let b = step("b");
    let graph = ExecutionGraph::new(
        PipelineId::new("fastq-to-fastq__default__v1").as_str(),
        "planner",
        PlanPolicy::default(),
        vec![a.clone(), b.clone()],
        vec![
            ExecutionEdge::new(a.step_id.clone(), b.step_id.clone()),
            ExecutionEdge::new(b.step_id, a.step_id),
        ],
    );
    assert!(graph.is_err());
}
