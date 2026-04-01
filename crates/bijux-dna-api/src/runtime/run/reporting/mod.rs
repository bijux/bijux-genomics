use super::{
    anyhow, build_run_execution_plan, ensure_stage_supported_by_runner, DockerRunner,
    DryRunRequest, DryRunResponse, ExecuteRequest, ExecuteResponse, Path, Profile, Result,
    RunExecutionPlan, RunId, RunSpec, RunnerContractKind, ToolRegistry,
};
use bijux_dna_engine::Engine;

mod lifecycle;
mod plan_response;
mod rendering;
mod replay;
mod status;
mod summary_artifact;
mod workspace_audit;

pub use plan_response::plan;
pub use rendering::{execute_and_report, render_report};
pub use replay::replay_manifest;
pub use status::status;
pub use workspace_audit::{policy_audit, workspace_edges, write_workspace_audit};

/// # Errors
/// Returns an error if execution fails.
pub fn execute(request: &ExecuteRequest) -> Result<ExecuteResponse> {
    let runner_contract = match request.runner {
        bijux_dna_environment::api::RuntimeKind::Docker => RunnerContractKind::Docker,
        other => return Err(anyhow!("runner {other} not supported for execute")),
    };
    for step in request.graph.steps() {
        ensure_stage_supported_by_runner(runner_contract, step.stage_id.as_str())?;
    }
    let (run_id, layout) = bijux_dna_runtime::run_layout::create_run_layout(&request.run_dir)?;
    let runner: Box<dyn bijux_dna_runtime::Runner> = match request.runner {
        bijux_dna_environment::api::RuntimeKind::Docker => Box::new(DockerRunner::new(None)),
        other => {
            return Err(anyhow!("runner {other} not supported for execute"));
        }
    };
    Engine::default().execute(&request.graph, runner.as_ref(), &layout, None, None)?;
    let summary_path = layout.summary_dir.join("run_summary.json");
    summary_artifact::write_run_summary_artifact(
        &summary_path,
        "execute",
        request.graph.pipeline_id().as_str(),
        &layout.manifest_path,
    )?;
    Ok(ExecuteResponse {
        run_id,
        manifest_path: layout.manifest_path,
        report_path: None,
    })
}

/// # Errors
/// Returns an error if dry-run output cannot be written.
pub fn dry_run(request: &DryRunRequest) -> Result<DryRunResponse> {
    let graph_hash = request.graph.hash()?;
    let graph_path = request.run_dir.join("graph.json");
    let graph_payload =
        bijux_dna_core::contract::canonical::to_canonical_json_bytes(&request.graph)?;
    bijux_dna_infra::atomic_write_bytes(&graph_path, graph_payload.as_slice())?;
    let mut manifest = serde_json::json!({
        "schema_version": "bijux.run_manifest.v3",
        "contract_version": bijux_dna_core::contract::ContractVersion::v1(),
        "run_id": "dry-run",
        "pipeline_id": request.graph.pipeline_id().to_string(),
        "profile_id": request.profile_id,
        "graph_hash": graph_hash,
        "cache_key": serde_json::Value::Null,
        "toolchain_versions": [],
        "dataset_fingerprints": [],
        "tool_invocations": [],
        "output_artifacts": [],
        "stages": [],
        "failures": [],
    });
    let manifest_path = request.run_dir.join("run_manifest.json");
    let payload = bijux_dna_core::contract::canonical::to_canonical_json_bytes(&manifest)?;
    bijux_dna_infra::atomic_write_bytes(&manifest_path, payload.as_slice())?;
    let summary_path = request.run_dir.join("run_summary.json");
    summary_artifact::write_run_summary_artifact(
        &summary_path,
        "dry-run",
        request.graph.pipeline_id().as_str(),
        &manifest_path,
    )?;
    let graph_sha = bijux_dna_infra::hash_file_sha256(&graph_path)?;
    let summary_sha = bijux_dna_infra::hash_file_sha256(&summary_path)?;
    manifest["output_artifacts"] = serde_json::json!([
        {
            "kind": "graph",
            "schema": "bijux.execution_graph.v1",
            "path": summary_artifact::relative_path_string(&request.run_dir, &graph_path),
            "sha256": graph_sha
        },
        {
            "kind": "run_summary",
            "schema": "bijux.run_summary.v1",
            "path": summary_artifact::relative_path_string(&request.run_dir, &summary_path),
            "sha256": summary_sha
        }
    ]);
    let payload = bijux_dna_core::contract::canonical::to_canonical_json_bytes(&manifest)?;
    bijux_dna_infra::atomic_write_bytes(&manifest_path, payload.as_slice())?;
    let manifest_sha = bijux_dna_infra::hash_file_sha256(&manifest_path)?;
    if let Some(artifacts) = manifest["output_artifacts"].as_array_mut() {
        artifacts.push(serde_json::json!({
            "kind": "run_manifest",
            "schema": "bijux.run_manifest.v3",
            "path": summary_artifact::relative_path_string(&request.run_dir, &manifest_path),
            "sha256": manifest_sha
        }));
    } else {
        return Err(anyhow!("dry-run manifest output_artifacts is not an array"));
    }
    let payload = bijux_dna_core::contract::canonical::to_canonical_json_bytes(&manifest)?;
    bijux_dna_infra::atomic_write_bytes(&manifest_path, payload.as_slice())?;
    bijux_dna_runtime::recording::write_profile_and_lock_manifests(&manifest_path)?;
    Ok(DryRunResponse {
        graph_path,
        manifest_path,
    })
}

/// # Errors
/// Returns an error if the tool registry or profile are invalid for the run spec.
#[allow(dead_code)]
pub fn build_stage_plan(
    run_spec: &RunSpec,
    registry: &ToolRegistry,
    profile: &Profile,
    run_id: RunId,
) -> Result<RunExecutionPlan> {
    build_run_execution_plan(run_spec, registry, profile, run_id)
}
