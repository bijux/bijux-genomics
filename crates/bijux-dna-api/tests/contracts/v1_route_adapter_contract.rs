use std::collections::BTreeSet;
use std::path::PathBuf;

use bijux_dna_api::v1::api::{route_version_inventory, DryRunResponse, ExecuteResponse, PlanResponse, RunStatus};

#[test]
fn route_inventory_exposes_governed_v1_adapters() {
    let inventory = route_version_inventory();
    assert_eq!(inventory.schema_version, "bijux.api_route_inventory.v1");
    assert_eq!(inventory.api_version, "v1");
    let route_ids = inventory.routes.iter().map(|route| route.route_id.as_str()).collect::<BTreeSet<_>>();
    assert_eq!(route_ids, BTreeSet::from(["v1.dry_run", "v1.execute", "v1.plan", "v1.status"]));
}

#[test]
fn plan_response_adapter_surfaces_workflow_and_plan_models() {
    let inventory = route_version_inventory();
    let plan = inventory
        .routes
        .iter()
        .find(|route| route.route_id == "v1.plan")
        .unwrap_or_else(|| panic!("missing v1.plan adapter"));
    assert_eq!(plan.response_struct, "PlanResponse");
    assert!(plan.writes_schema_families.contains(&"workflow_manifest".to_string()));
    assert!(plan.writes_schema_families.contains(&"plan_manifest".to_string()));
    let type_name = std::any::type_name::<PlanResponse>();
    assert!(type_name.ends_with("PlanResponse"));
}

#[test]
fn execute_and_status_adapters_match_runtime_and_evidence_fields() {
    let inventory = route_version_inventory();
    let execute = inventory
        .routes
        .iter()
        .find(|route| route.route_id == "v1.execute")
        .unwrap_or_else(|| panic!("missing v1.execute adapter"));
    let status = inventory
        .routes
        .iter()
        .find(|route| route.route_id == "v1.status")
        .unwrap_or_else(|| panic!("missing v1.status adapter"));

    assert!(execute.writes_schema_families.contains(&"run_state".to_string()));
    assert!(execute.writes_schema_families.contains(&"evidence_bundle".to_string()));
    assert!(status.reads_schema_families.contains(&"artifact_inventory".to_string()));
    assert!(status.reads_schema_families.contains(&"evidence_verification".to_string()));

    let execute_json = serde_json::to_value(ExecuteResponse {
        run_id: "run-1".to_string(),
        correlation_id: "enforced:run-1".to_string(),
        manifest_path: PathBuf::from("run_manifest.json"),
        run_state_path: PathBuf::from("run_state.json"),
        runtime_policy_path: PathBuf::from("runtime_policy.json"),
        executor_descriptor_path: PathBuf::from("executor_descriptor.json"),
        checkpoint_path: PathBuf::from("checkpoints/checkpoint.json"),
        failure_path: Some(PathBuf::from("run_failure.json")),
        mode: bijux_dna_runtime::run_layout::RunExecutionModeV1::Enforced,
        state: bijux_dna_runtime::run_layout::RunLifecycleStateV1::Succeeded,
        report_path: Some(PathBuf::from("report.json")),
        evidence_bundle_path: PathBuf::from("evidence_bundle.json"),
        evidence_verification_path: PathBuf::from("evidence_verification.json"),
        artifact_inventory_path: PathBuf::from("artifact_inventory.json"),
        replay_manifest_path: PathBuf::from("replay_manifest.json"),
        hash_ledger_path: PathBuf::from("hash_ledger.json"),
        run_summary_text_path: PathBuf::from("summary/run_summary.txt"),
    })
    .unwrap_or_else(|err| panic!("serialize ExecuteResponse: {err}"));
    for key in [
        "run_state_path",
        "runtime_policy_path",
        "artifact_inventory_path",
        "evidence_bundle_path",
        "evidence_verification_path",
    ] {
        assert!(execute_json.get(key).is_some(), "execute response missing {key}");
    }

    let status = RunStatus {
        run_dir: PathBuf::from("run"),
        manifest_path: Some(PathBuf::from("run_manifest.json")),
        report_path: Some(PathBuf::from("report.json")),
        evidence_bundle_path: Some(PathBuf::from("evidence_bundle.json")),
        evidence_verification_path: Some(PathBuf::from("evidence_verification.json")),
        artifact_inventory_path: Some(PathBuf::from("artifact_inventory.json")),
        artifact_inventory_text_path: Some(PathBuf::from("artifact_inventory.txt")),
        replay_manifest_path: Some(PathBuf::from("replay_manifest.json")),
        hash_ledger_path: Some(PathBuf::from("hash_ledger.json")),
        run_summary_text_path: Some(PathBuf::from("summary/run_summary.txt")),
        run_state_path: Some(PathBuf::from("run_state.json")),
        runtime_policy_path: Some(PathBuf::from("runtime_policy.json")),
        executor_descriptor_path: Some(PathBuf::from("executor_descriptor.json")),
        checkpoint_path: Some(PathBuf::from("checkpoints/checkpoint.json")),
        failure_path: None,
        correlation_id: Some("enforced:run-1".to_string()),
        mode: Some(bijux_dna_runtime::run_layout::RunExecutionModeV1::Enforced),
        state: Some(bijux_dna_runtime::run_layout::RunLifecycleStateV1::Succeeded),
        has_failures: false,
    };
    let status_json = serde_json::json!({
        "run_dir": status.run_dir,
        "manifest_path": status.manifest_path,
        "report_path": status.report_path,
        "evidence_bundle_path": status.evidence_bundle_path,
        "evidence_verification_path": status.evidence_verification_path,
        "artifact_inventory_path": status.artifact_inventory_path,
        "artifact_inventory_text_path": status.artifact_inventory_text_path,
        "replay_manifest_path": status.replay_manifest_path,
        "hash_ledger_path": status.hash_ledger_path,
        "run_summary_text_path": status.run_summary_text_path,
        "run_state_path": status.run_state_path,
        "runtime_policy_path": status.runtime_policy_path,
        "executor_descriptor_path": status.executor_descriptor_path,
        "checkpoint_path": status.checkpoint_path,
        "failure_path": status.failure_path,
        "correlation_id": status.correlation_id,
        "mode": status.mode,
        "state": status.state,
        "has_failures": status.has_failures,
    });
    for key in [
        "artifact_inventory_path",
        "evidence_bundle_path",
        "evidence_verification_path",
        "run_state_path",
        "runtime_policy_path",
    ] {
        assert!(status_json.get(key).is_some(), "run status missing {key}");
    }
}

#[test]
fn dry_run_adapter_declares_runtime_and_evidence_outputs() {
    let inventory = route_version_inventory();
    let dry_run = inventory
        .routes
        .iter()
        .find(|route| route.route_id == "v1.dry_run")
        .unwrap_or_else(|| panic!("missing v1.dry_run adapter"));
    assert_eq!(dry_run.response_struct, "DryRunResponse");
    assert!(dry_run.writes_schema_families.contains(&"run_state".to_string()));
    assert!(dry_run.writes_schema_families.contains(&"artifact_inventory".to_string()));
    assert!(std::any::type_name::<DryRunResponse>().ends_with("DryRunResponse"));
}
