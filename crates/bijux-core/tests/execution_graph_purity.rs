use bijux_core::contract::PlanPolicy;
use bijux_core::contract::{ArtifactRef, ArtifactRole, StageIO, ToolConstraints};
use bijux_core::contract::{ExecutionGraph, ExecutionStep};
use bijux_core::{ArtifactId, CommandSpecV1, ContainerImageRefV1, StageId, StepId};

#[test]
fn execution_graph_serialization_is_stage_plan_free() {
    let step = ExecutionStep {
        step_id: StepId::new("fastq.validate_pre"),
        stage_id: StageId::new("fastq.validate_pre"),
        command: CommandSpecV1 {
            template: vec!["tool".to_string()],
        },
        image: ContainerImageRefV1 {
            image: "tool".to_string(),
            digest: None,
        },
        resources: ToolConstraints::default(),
        io: StageIO {
            inputs: vec![ArtifactRef::required(
                ArtifactId::from_static("input"),
                "input.fastq".into(),
                ArtifactRole::Reads,
            )],
            outputs: vec![ArtifactRef::required(
                ArtifactId::from_static("output"),
                "output.fastq".into(),
                ArtifactRole::Reads,
            )],
        },
        out_dir: "out".into(),
        aux_images: std::collections::BTreeMap::new(),
        expected_artifact_ids: Vec::new(),
        metrics_schema_ids: Vec::new(),
    };
    let graph = ExecutionGraph::new(
        "fastq-to-fastq__default__v1",
        "planner",
        PlanPolicy::PreferAccuracy,
        vec![step],
        Vec::new(),
    );
    let graph = graph.expect("graph");
    let encoded = serde_json::to_string(&graph).expect("serialize");
    assert!(!encoded.contains("StagePlanV1"));
    assert!(!encoded.contains("StagePlugin"));
    assert!(!encoded.contains("stage_plugin"));
}
