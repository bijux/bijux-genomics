/// Snapshot intent: verifies stable, reviewed output for this contract.
use std::collections::BTreeMap;
use std::path::PathBuf;

use bijux_dna_api::v1::api::{
    dry_run, explain, plan, policy_audit, status, DryRunRequest, ExecuteResponse, PlanRequest,
};
use bijux_dna_core::contract::PlanPolicy;
use bijux_dna_core::contract::{ArtifactRef, ArtifactRole, StageIO, ToolConstraints};
use bijux_dna_core::contract::{ExecutionEdge, ExecutionGraph, ExecutionStep};
use bijux_dna_core::prelude::{ArtifactId, CommandSpecV1, ContainerImageRefV1, StageId, StepId};
use insta::Settings;

fn snapshot_name(group: &str, name: &str) -> String {
    format!("bijux-dna-api__{group}__{name}")
}

fn minimal_graph() -> ExecutionGraph {
    let step = ExecutionStep {
        step_id: StepId::from_static("core.test"),
        stage_id: StageId::from_static("core.test"),
        image: ContainerImageRefV1 {
            image: "example/tool:1.0".to_string(),
            digest: Some("sha256:deadbeef".to_string()),
        },
        command: CommandSpecV1 { template: vec!["echo".to_string(), "hello".to_string()] },
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
    .unwrap_or_else(|err| panic!("graph build failed: {err}"))
}

fn snapshot_settings() -> Settings {
    let mut settings = Settings::new();
    settings.set_prepend_module_to_snapshot(false);
    settings.set_snapshot_path(
        crate::support::crate_snapshots("bijux-dna-api")
            .unwrap_or_else(|err| panic!("resolve snapshots root: {err}")),
    );
    settings
}

#[test]
fn plan_response_schema_is_stable() -> anyhow::Result<()> {
    let graph = minimal_graph();
    let request = PlanRequest {
        graph,
        profile_id: "default".to_string(),
        workflow_manifest: None,
        stage_plans: Vec::new(),
        parameter_traces: Vec::new(),
        planner_refusals: Vec::new(),
        planner_warnings: Vec::new(),
        compare_against: None,
    };
    let response = plan(request)?;
    let json = serde_json::to_value(&response)?;
    let name = snapshot_name("schemas", "plan_response_schema");
    let settings = snapshot_settings();
    settings.bind(|| {
        insta::assert_json_snapshot!(name, bijux_dna_testkit::snapshot_normalize_json(&json));
    });
    Ok(())
}

#[test]
fn execute_response_schema_is_stable() -> anyhow::Result<()> {
    let response = ExecuteResponse {
        run_id: "run-1".to_string(),
        correlation_id: "run:run-1".to_string(),
        manifest_path: PathBuf::from("runs/run-1/run_manifest.json"),
        report_path: Some(PathBuf::from("runs/run-1/run_artifacts/report.html")),
        evidence_bundle_path: PathBuf::from("runs/run-1/evidence_bundle.json"),
    };
    let json = serde_json::to_value(&response)?;
    let name = snapshot_name("schemas", "execute_response_schema");
    let settings = snapshot_settings();
    settings.bind(|| {
        insta::assert_json_snapshot!(name, bijux_dna_testkit::snapshot_normalize_json(&json));
    });
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
    let root = temp
        .path()
        .to_str()
        .unwrap_or_else(|| panic!("temp root is not valid UTF-8: {}", temp.path().display()));
    scrub_paths(&mut json, root);
    let name = snapshot_name("schemas", "dry_run_response_schema");
    let settings = snapshot_settings();
    settings.bind(|| {
        insta::assert_json_snapshot!(name, bijux_dna_testkit::snapshot_normalize_json(&json));
    });
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
        "evidence_bundle_path": status.evidence_bundle_path,
        "correlation_id": status.correlation_id,
        "has_failures": status.has_failures,
    });
    let root = temp
        .path()
        .to_str()
        .unwrap_or_else(|| panic!("temp root is not valid UTF-8: {}", temp.path().display()));
    scrub_paths(&mut json, root);
    let name = snapshot_name("schemas", "status_schema");
    let settings = snapshot_settings();
    settings.bind(|| {
        insta::assert_json_snapshot!(name, bijux_dna_testkit::snapshot_normalize_json(&json));
    });
    Ok(())
}

#[test]
fn explain_schema_is_stable() -> anyhow::Result<()> {
    let graph = minimal_graph();
    let response = explain(&graph, None);
    let json = serde_json::to_value(&response)?;
    let name = snapshot_name("schemas", "explain_schema");
    let settings = snapshot_settings();
    settings.bind(|| {
        insta::assert_json_snapshot!(name, bijux_dna_testkit::snapshot_normalize_json(&json));
    });
    Ok(())
}

#[test]
fn policy_audit_schema_is_stable() -> anyhow::Result<()> {
    let json = policy_audit()?;
    let name = snapshot_name("schemas", "policy_audit_schema");
    let settings = snapshot_settings();
    settings.bind(|| {
        insta::assert_json_snapshot!(name, bijux_dna_testkit::snapshot_normalize_json(&json));
    });
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
