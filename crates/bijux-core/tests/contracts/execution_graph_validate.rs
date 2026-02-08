use std::collections::BTreeMap;
use std::path::PathBuf;

use bijux_core::contract::execution::{ExecutionEdge, ExecutionGraph, ExecutionStep};
use bijux_core::contract::{ArtifactRole, PlanPolicy, StageIO, ToolConstraints};
use bijux_core::prelude::{
    ArtifactId, CommandSpecV1, ContainerImageRefV1, PipelineId, StageId, StepId,
};

fn step(step_id: &str) -> ExecutionStep {
    ExecutionStep {
        step_id: StepId::new(step_id),
        stage_id: StageId::new("fastq.trim"),
        command: CommandSpecV1 {
            template: vec!["fastp".to_string()],
        },
        image: ContainerImageRefV1 {
            image: "fastp:latest".to_string(),
            digest: None,
        },
        resources: ToolConstraints::default(),
        io: StageIO {
            inputs: vec![bijux_core::contract::ArtifactSpec::required(
                ArtifactId::new("in"),
                PathBuf::from("in"),
                ArtifactRole::Reads,
            )],
            outputs: vec![bijux_core::contract::ArtifactSpec::required(
                ArtifactId::new("out"),
                PathBuf::from("out"),
                ArtifactRole::Reads,
            )],
        },
        out_dir: PathBuf::from("/tmp"),
        aux_images: BTreeMap::default(),
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
