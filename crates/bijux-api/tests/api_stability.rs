use std::collections::BTreeMap;
use std::path::PathBuf;

use bijux_api::v1::api::{
    dry_run, explain, plan, policy_audit, status, DryRunRequest, ExecuteResponse, PlanRequest,
};
use bijux_core::contract::PlanPolicy;
use bijux_core::contract::{ArtifactRef, ArtifactRole, StageIO, ToolConstraints};
use bijux_core::contract::{ExecutionEdge, ExecutionGraph, ExecutionStep};
use bijux_core::{ArtifactId, CommandSpecV1, ContainerImageRefV1, StageId, StepId};
use tempfile;

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

#[test]
fn dry_run_response_schema_is_stable() -> anyhow::Result<()> {
    let graph = minimal_graph();
    let temp = tempfile::tempdir()?;
    let request = DryRunRequest {
        graph,
        run_dir: temp.path().to_path_buf(),
        profile_id: "default".to_string(),
    };
    let response = dry_run(&request)?;
    let mut json = serde_json::to_value(&response)?;
    let root = temp.path().to_str().unwrap_or_default();
    scrub_paths(&mut json, root);
    insta::assert_json_snapshot!("dry_run_response_schema", json);
    Ok(())
}

#[test]
fn status_schema_is_stable() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    let manifest_path = temp.path().join("run_manifest.json");
    std::fs::write(&manifest_path, "{}")?;
    let status = status(temp.path())?;
    let mut json = serde_json::json!({
        "run_dir": status.run_dir,
        "manifest_path": status.manifest_path,
        "report_path": status.report_path,
        "has_failures": status.has_failures,
    });
    let root = temp.path().to_str().unwrap_or_default();
    scrub_paths(&mut json, root);
    insta::assert_json_snapshot!("status_schema", json);
    Ok(())
}

#[test]
fn explain_schema_is_stable() -> anyhow::Result<()> {
    let graph = minimal_graph();
    let response = explain(&graph, None);
    let json = serde_json::to_value(&response)?;
    insta::assert_json_snapshot!("explain_schema", json);
    Ok(())
}

#[test]
fn policy_audit_schema_is_stable() -> anyhow::Result<()> {
    let mut json = policy_audit()?;
    if let Some(guardrails) = json.get_mut("guardrails") {
        if let Some(obj) = guardrails.as_object_mut() {
            for value in obj.values_mut() {
                if let Some(error) = value.get_mut("error") {
                    if !error.is_null() {
                        *error = serde_json::Value::String("<error>".to_string());
                    }
                }
            }
        }
    }
    insta::assert_json_snapshot!("policy_audit_schema", json);
    Ok(())
}

fn scrub_paths(value: &mut serde_json::Value, root: &str) {
    match value {
        serde_json::Value::String(s) => {
            if s.contains(root) {
                *s = s.replace(root, "<temp>");
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                scrub_paths(item, root);
            }
        }
        serde_json::Value::Object(map) => {
            for value in map.values_mut() {
                scrub_paths(value, root);
            }
        }
        _ => {}
    }
}
