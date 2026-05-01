use super::operations::{build_health_report, request_control_action};
use super::Result;
use crate::request_args::{OperatorHealthResponse, RunControlResponse};
use anyhow::{anyhow, Context};
use bijux_dna_environment::api::RuntimeKind;
use bijux_dna_runtime::run_layout::{
    ExecutorDescriptorV1, RunControlActionV1, RunControlStateV1, RunExecutorDescriptorV1, RunLayout,
};
use std::path::Path;

/// # Errors
/// Returns an error if the control state cannot be written.
pub fn pause_run(run_dir: &Path) -> Result<RunControlResponse> {
    set_run_control(run_dir, RunControlActionV1::Pause, "operator requested pause")
}

/// # Errors
/// Returns an error if the control state cannot be written.
pub fn resume_run(run_dir: &Path) -> Result<RunControlResponse> {
    set_run_control(run_dir, RunControlActionV1::Resume, "operator requested resume")
}

/// # Errors
/// Returns an error if the control state cannot be written.
pub fn cancel_run(run_dir: &Path) -> Result<RunControlResponse> {
    set_run_control(run_dir, RunControlActionV1::Cancel, "operator requested cancel")
}

/// # Errors
/// Returns an error if the health report cannot be materialized.
pub fn operator_health(run_dir: &Path) -> Result<OperatorHealthResponse> {
    let layout = RunLayout::from_run_dir(run_dir.to_path_buf());
    let run_id = infer_run_id(&layout);
    let runner = infer_runtime_kind(&layout)?;
    let report = build_health_report(&layout, &run_id, runner);
    bijux_dna_runtime::run_layout::write_health_report(&layout, &report)?;
    Ok(OperatorHealthResponse { health_report_path: layout.health_report_path, report })
}

fn set_run_control(
    run_dir: &Path,
    action: RunControlActionV1,
    detail: &str,
) -> Result<RunControlResponse> {
    let layout = RunLayout::from_run_dir(run_dir.to_path_buf());
    let run_id = infer_run_id(&layout);
    let state = request_control_action(&layout, &run_id, action, detail)?;
    Ok(RunControlResponse {
        control_state_path: layout.control_state_path,
        queue_state_path: layout.queue_state_path.exists().then_some(layout.queue_state_path),
        state,
    })
}

fn infer_run_id(layout: &RunLayout) -> String {
    read_run_control(layout)
        .map(|state| state.run_id)
        .or_else(|| {
            if !layout.run_state_path.exists() {
                return None;
            }
            let raw = std::fs::read_to_string(&layout.run_state_path).ok()?;
            serde_json::from_str::<bijux_dna_runtime::run_layout::RunStateV1>(&raw)
                .ok()
                .map(|state| state.run_id)
        })
        .unwrap_or_else(|| {
            layout.run_dir.file_name().and_then(|value| value.to_str()).unwrap_or("run").to_string()
        })
}

fn infer_runtime_kind(layout: &RunLayout) -> Result<RuntimeKind> {
    if !layout.executor_descriptor_path.exists() {
        return Ok(RuntimeKind::Local);
    }
    let raw = std::fs::read_to_string(&layout.executor_descriptor_path)
        .with_context(|| format!("read {}", layout.executor_descriptor_path.display()))?;
    let descriptor: RunExecutorDescriptorV1 = serde_json::from_str(&raw)
        .with_context(|| format!("parse {}", layout.executor_descriptor_path.display()))?;
    match descriptor.descriptor {
        ExecutorDescriptorV1::Local { .. } => Ok(RuntimeKind::Local),
        ExecutorDescriptorV1::Container { runtime, .. } => match runtime.as_str() {
            "docker" => Ok(RuntimeKind::Docker),
            "apptainer" => Ok(RuntimeKind::Apptainer),
            "singularity" => Ok(RuntimeKind::Singularity),
            other => Err(anyhow!("unsupported runtime descriptor `{other}`")),
        },
        ExecutorDescriptorV1::Hpc { container_runtime, .. } => match container_runtime.as_deref() {
            Some("apptainer") => Ok(RuntimeKind::Apptainer),
            Some("singularity") => Ok(RuntimeKind::Singularity),
            Some("docker") => Ok(RuntimeKind::Docker),
            Some(other) => Err(anyhow!("unsupported hpc container runtime `{other}`")),
            None => Ok(RuntimeKind::Local),
        },
    }
}

fn read_run_control(layout: &RunLayout) -> Option<RunControlStateV1> {
    if !layout.control_state_path.exists() {
        return None;
    }
    let raw = std::fs::read_to_string(&layout.control_state_path).ok()?;
    serde_json::from_str(&raw).ok()
}
