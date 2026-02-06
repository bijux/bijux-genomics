use bijux_api::v1::plan::PlanExplainV1;
use bijux_core::contract::{ArtifactRef, ArtifactRole, StageIO, ToolConstraints};
use bijux_core::execution::execution_graph::{ExecutionEdge, ExecutionGraph, ExecutionStep};
use bijux_core::execution::PlanPolicy;
use bijux_core::{CommandSpecV1, ContainerImageRefV1, StageId};
use std::path::PathBuf;

#[test]
fn explain_roundtrip_is_deterministic() -> anyhow::Result<()> {
    let stage = ExecutionStep {
        step_id: StageId::from_static("stage.a"),
        image: ContainerImageRefV1 {
            image: "tool:1.0.0".to_string(),
            digest: None,
        },
        command: CommandSpecV1 {
            template: vec!["tool".to_string()],
        },
        resources: ToolConstraints {
            runtime: "docker".to_string(),
            mem_gb: 1,
            tmp_gb: 1,
            threads: 1,
        },
        io: StageIO {
            inputs: vec![ArtifactRef::required(
                "input",
                PathBuf::from("input"),
                ArtifactRole::Unknown,
            )],
            outputs: vec![ArtifactRef::required(
                "output",
                PathBuf::from("output"),
                ArtifactRole::Unknown,
            )],
        },
        out_dir: PathBuf::from("out"),
        aux_images: std::collections::BTreeMap::new(),
        expected_artifact_ids: Vec::new(),
        metrics_schema_ids: Vec::new(),
    };
    let plan = ExecutionGraph::new(
        "pipeline",
        "planner",
        PlanPolicy::PreferAccuracy,
        vec![stage],
        Vec::<ExecutionEdge>::new(),
    )?;

    let explain = PlanExplainV1::from_plan(&plan);
    let json = serde_json::to_string(&explain)?;
    let parsed: PlanExplainV1 = serde_json::from_str(&json)?;
    let json_roundtrip = serde_json::to_string(&parsed)?;
    assert_eq!(json, json_roundtrip);

    let explain_again = PlanExplainV1::from_plan(&plan);
    assert_eq!(explain, explain_again);
    Ok(())
}
