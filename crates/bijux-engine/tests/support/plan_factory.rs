use std::collections::BTreeMap;
use std::path::PathBuf;

use bijux_core::contract::{
    ArtifactRef, ArtifactRole, ExecutionEdge, ExecutionGraph, ExecutionStep, PlanPolicy, StageIO,
    ToolConstraints,
};
use bijux_core::prelude::{ArtifactId, CommandSpecV1, ContainerImageRefV1, StageId, StepId};

pub fn plan_for(stage_id: &str) -> ExecutionStep {
    let out_dir = tempfile::Builder::new()
        .prefix(&format!("bijux-engine-test-{stage_id}-"))
        .tempdir()
        .unwrap_or_else(|err| panic!("tempdir: {err}"))
        .keep();
    ExecutionStep {
        step_id: StepId::new(stage_id),
        stage_id: StageId::new(stage_id),
        image: ContainerImageRefV1 {
            image: "tool".to_string(),
            digest: Some("sha256:img".to_string()),
        },
        command: CommandSpecV1 {
            template: vec!["tool".to_string()],
        },
        resources: ToolConstraints {
            runtime: "short".to_string(),
            mem_gb: 1,
            tmp_gb: 1,
            threads: 1,
        },
        io: StageIO {
            inputs: vec![ArtifactRef::required(
                ArtifactId::from_static("input"),
                PathBuf::from("input"),
                ArtifactRole::Unknown,
            )],
            outputs: vec![ArtifactRef::optional(
                ArtifactId::from_static("output"),
                PathBuf::from("output"),
                ArtifactRole::Unknown,
            )],
        },
        out_dir,
        aux_images: BTreeMap::new(),
        expected_artifact_ids: Vec::new(),
        metrics_schema_ids: Vec::new(),
    }
}

pub fn build_graph(stages: Vec<ExecutionStep>, edges: Vec<ExecutionEdge>) -> ExecutionGraph {
    ExecutionGraph::new(
        "fastq-to-fastq__default__v1",
        "planner",
        PlanPolicy::PreferAccuracy,
        stages,
        edges,
    )
    .unwrap_or_else(|err| panic!("plan: {err}"))
}
