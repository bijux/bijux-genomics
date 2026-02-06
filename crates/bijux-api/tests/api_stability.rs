use std::collections::BTreeMap;
use std::path::PathBuf;

use bijux_api::v1::run::{plan, ExecuteResponse, PlanRequest};
use bijux_core::contract::{ArtifactRef, ArtifactRole, StageIO, ToolConstraints};
use bijux_core::execution::execution_graph::{ExecutionEdge, ExecutionGraph, ExecutionStep};
use bijux_core::execution::PlanPolicy;
use bijux_core::{ArtifactId, CommandSpecV1, ContainerImageRefV1, StageId, StepId};

fn minimal_graph() -> ExecutionGraph {
    let step = ExecutionStep {
        step_id: StepId::from_static("core.test"),
        stage_id: StageId::from_static("core.test"),
        image: ContainerImageRefV1 {
            image: "example/tool:1.0".to_string(),
            digest: Some("sha256:deadbeef".to_string()),
        },
        command: CommandSpecV1 {
            template: vec!["echo".to_string(), "hello".to_string()],
        },
        resources: ToolConstraints::default(),
        io: StageIO {
            inputs: vec![ArtifactRef::required(
                ArtifactId::from_static("input"),
                PathBuf::from("input"),
                ArtifactRole::Reads,
            )],
            outputs: vec![ArtifactRef::required(
                ArtifactId::from_static("output"),
                PathBuf::from("output"),
                ArtifactRole::Reads,
            )],
        },
        out_dir: PathBuf::from("out"),
        aux_images: BTreeMap::new(),
        expected_artifact_ids: Vec::new(),
        metrics_schema_ids: Vec::new(),
    };
    ExecutionGraph::new(
        "fastq-to-fastq__default__v1",
        "planner.test",
        PlanPolicy::PreferAccuracy,
        vec![step],
        Vec::<ExecutionEdge>::new(),
    )
    .expect("graph")
}

#[test]
fn plan_response_schema_is_stable() -> anyhow::Result<()> {
    let graph = minimal_graph();
    let request = PlanRequest {
        graph,
        profile_id: "default".to_string(),
    };
    let response = plan(request)?;
    let json = serde_json::to_value(&response)?;
    insta::assert_json_snapshot!("plan_response_schema", json);
    Ok(())
}

#[test]
fn execute_response_schema_is_stable() -> anyhow::Result<()> {
    let response = ExecuteResponse {
        run_id: "run-1".to_string(),
        manifest_path: PathBuf::from("runs/run-1/run_manifest.json"),
        report_path: Some(PathBuf::from("runs/run-1/run_artifacts/report.html")),
    };
    let json = serde_json::to_value(&response)?;
    insta::assert_json_snapshot!("execute_response_schema", json);
    Ok(())
}
