use super::{summary_artifact, Result};
use crate::request_args::{ExecuteRequest, ExecuteResponse};
use anyhow::anyhow;
use bijux_dna_engine::Engine;
use bijux_dna_runner::DockerRunner;
use bijux_dna_runtime::{ensure_stage_supported_by_runner, RunnerContractKind};

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
    let correlation_id = format!("run:{run_id}");
    let evidence_bundle_path = bijux_dna_analyze::write_evidence_bundle_json(&layout.run_dir, None)?;
    summary_artifact::attach_output_artifact(
        &layout.manifest_path,
        &layout.run_dir,
        &correlation_id,
        "evidence_bundle",
        "bijux.evidence_bundle.v1",
        &evidence_bundle_path,
    )?;
    Ok(ExecuteResponse {
        run_id,
        correlation_id,
        manifest_path: layout.manifest_path,
        report_path: None,
        evidence_bundle_path,
    })
}
