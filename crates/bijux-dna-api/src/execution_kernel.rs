use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{anyhow, bail, Context, Result};
use bijux_dna_core::contract::ExecutionStep;
use bijux_dna_environment::api::RuntimeKind;
use bijux_dna_runner::execute::{execute_step, StageResultV1};
use bijux_dna_runtime::recording::{hash_file_sha256, write_execution_logs_bounded};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkPolicy {
    Allow,
    Forbid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolContext {
    pub run_id: String,
    pub stage_id: String,
    pub tool_id: String,
    pub sample_id: Option<String>,
    pub stage_root: PathBuf,
    pub input_root: PathBuf,
    pub output_root: PathBuf,
    pub tmp_root: PathBuf,
    pub threads: u32,
    pub memory_hint_mb: Option<u64>,
    pub seed: Option<u64>,
    pub network_policy: NetworkPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInvocationRequest {
    pub step: ExecutionStep,
    pub runner: RuntimeKind,
    pub context: ToolContext,
    pub timeout: Option<Duration>,
}

#[derive(Debug, Clone)]
pub struct ToolInvocationResult {
    pub stage_result: StageResultV1,
    pub runtime_provenance_path: PathBuf,
    pub stage_manifest_path: PathBuf,
    pub stdout_path: PathBuf,
    pub stderr_path: PathBuf,
    pub summary_path: PathBuf,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ExitTaxonomy {
    ToolFailure,
    UserError,
    ContractViolation,
}

fn canonicalize_existing(path: &Path) -> Result<PathBuf> {
    if path.exists() {
        return path
            .canonicalize()
            .with_context(|| format!("canonicalize {}", path.display()));
    }
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(std::env::current_dir()
            .context("resolve cwd for relative path contracts")?
            .join(path))
    }
}

fn ensure_subpath(path: &Path, root: &Path, label: &str) -> Result<()> {
    let cpath = canonicalize_existing(path)?;
    let croot = canonicalize_existing(root)?;
    if !cpath.starts_with(&croot) {
        bail!(
            "{label} path contract violated: {} not under {}",
            cpath.display(),
            croot.display()
        );
    }
    Ok(())
}

fn enforce_path_contracts(req: &ToolInvocationRequest) -> Result<()> {
    ensure_subpath(&req.context.stage_root, &req.context.output_root, "stage_root")?;
    ensure_subpath(&req.context.tmp_root, &req.context.output_root, "tmp_root")?;
    for artifact in &req.step.io.outputs {
        ensure_subpath(&artifact.path, &req.context.output_root, "output")?;
    }
    for artifact in &req.step.io.inputs {
        ensure_subpath(&artifact.path, &req.context.input_root, "input")?;
    }
    Ok(())
}

fn classify_exit_code(exit_code: i32) -> ExitTaxonomy {
    match exit_code {
        0 => ExitTaxonomy::ToolFailure,
        2 | 64..=78 => ExitTaxonomy::UserError,
        126 | 127 => ExitTaxonomy::ContractViolation,
        _ => ExitTaxonomy::ToolFailure,
    }
}

fn network_policy_violation(policy: &NetworkPolicy) -> bool {
    matches!(policy, NetworkPolicy::Forbid)
        && std::env::var("BIJUX_ALLOW_NETWORK")
            .ok()
            .is_some_and(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
}

fn output_checksums(step: &ExecutionStep) -> BTreeMap<String, String> {
    let mut checksums = BTreeMap::new();
    for artifact in &step.io.outputs {
        if artifact.path.exists() {
            if let Ok(sum) = hash_file_sha256(&artifact.path) {
                checksums.insert(artifact.name.to_string(), sum);
            }
        }
    }
    checksums
}

/// # Errors
/// Returns an error if path contracts fail, tool execution fails, or artifacts cannot be written.
pub fn invoke_tool(req: &ToolInvocationRequest) -> Result<ToolInvocationResult> {
    enforce_path_contracts(req)?;
    if network_policy_violation(&req.context.network_policy) {
        bail!(
            "network policy violation: {} forbids network but BIJUX_ALLOW_NETWORK is enabled",
            req.context.stage_id
        );
    }
    bijux_dna_infra::ensure_dir(&req.context.stage_root)?;
    bijux_dna_infra::ensure_dir(&req.context.tmp_root)?;
    std::env::set_var("TMPDIR", &req.context.tmp_root);

    let logs_dir = req.context.stage_root.join("logs");
    bijux_dna_infra::ensure_dir(&logs_dir)?;
    let started_at = chrono::Utc::now();
    let stage_log_path = req.context.stage_root.join("stage_events.jsonl");
    let start_event = serde_json::json!({
        "schema_version": "bijux.stage_events.v1",
        "event": "tool_start",
        "run_id": req.context.run_id,
        "stage_id": req.context.stage_id,
        "tool_id": req.context.tool_id,
        "timestamp": started_at,
    });
    bijux_dna_runtime::recording::append_jsonl_line(
        &stage_log_path,
        &serde_json::to_string(&start_event)?,
    )?;

    let stage_result = execute_step(&req.step, req.runner, req.timeout)?;
    let log_paths = write_execution_logs_bounded(&logs_dir, &stage_result.stdout, &stage_result.stderr)
        .context("write stage logs")?;
    let stdout_path = logs_dir.join("tool.stdout.log");
    let stderr_path = logs_dir.join("tool.stderr.log");
    let stderr_tail = std::fs::read_to_string(&stderr_path).unwrap_or_default();
    let stdout_tail = std::fs::read_to_string(&stdout_path).unwrap_or_default();

    let exit_taxonomy = classify_exit_code(stage_result.exit_code);
    if stage_result.exit_code != 0 {
        let message = format!(
            "tool {} failed with exit code {} ({exit_taxonomy:?})\nstdout_tail:\n{}\nstderr_tail:\n{}",
            req.context.tool_id,
            stage_result.exit_code,
            stdout_tail,
            stderr_tail
        );
        return Err(anyhow!(message));
    }

    let finished_at = chrono::Utc::now();
    let runtime_provenance_path = req.context.stage_root.join("runtime_provenance.json");
    let env_summary = serde_json::json!({
        "hostname": std::env::var("HOSTNAME").ok(),
        "tz": std::env::var("TZ").ok(),
        "lc_all": std::env::var("LC_ALL").ok(),
    });
    let runtime_provenance = serde_json::json!({
        "schema_version": "bijux.runtime_provenance.v1",
        "run_id": req.context.run_id,
        "stage_id": req.context.stage_id,
        "tool_id": req.context.tool_id,
        "runner": req.runner.to_string(),
        "image": req.step.image.image,
        "tool_digest": req.step.image.digest,
        "tool_version": "unknown",
        "command": stage_result.command,
        "env_summary": env_summary,
        "started_at": started_at,
        "finished_at": finished_at,
        "exit_code": stage_result.exit_code,
    });
    bijux_dna_infra::atomic_write_json(&runtime_provenance_path, &runtime_provenance)?;

    let stage_manifest_path = req.context.stage_root.join("stage_manifest.json");
    let stage_manifest = serde_json::json!({
        "schema_version": "bijux.stage_manifest.v1",
        "run_id": req.context.run_id,
        "stage_id": req.context.stage_id,
        "tool_id": req.context.tool_id,
        "sample_id": req.context.sample_id,
        "threads": req.context.threads,
        "memory_hint_mb": req.context.memory_hint_mb,
        "seed": req.context.seed,
        "network_policy": req.context.network_policy,
        "inputs": req.step.io.inputs,
        "outputs": req.step.io.outputs,
        "output_checksums": output_checksums(&req.step),
        "runtime": {
            "runtime_s": stage_result.runtime_s,
            "memory_mb": stage_result.memory_mb,
            "exit_code": stage_result.exit_code,
        },
        "logs": log_paths,
        "runtime_provenance": runtime_provenance_path,
    });
    bijux_dna_infra::atomic_write_json(&stage_manifest_path, &stage_manifest)?;

    let end_event = serde_json::json!({
        "schema_version": "bijux.stage_events.v1",
        "event": "tool_end",
        "run_id": req.context.run_id,
        "stage_id": req.context.stage_id,
        "tool_id": req.context.tool_id,
        "timestamp": finished_at,
        "runtime_s": stage_result.runtime_s,
        "exit_code": stage_result.exit_code,
    });
    bijux_dna_runtime::recording::append_jsonl_line(
        &stage_log_path,
        &serde_json::to_string(&end_event)?,
    )?;

    let summary_path = req.context.stage_root.join("stage_human_summary.json");
    let summary = serde_json::json!({
        "schema_version": "bijux.stage_summary.v1",
        "stage_id": req.context.stage_id,
        "tool_id": req.context.tool_id,
        "status": "ok",
        "runtime_s": stage_result.runtime_s,
        "exit_code": stage_result.exit_code,
        "stdout_path": stdout_path,
        "stderr_path": stderr_path,
    });
    bijux_dna_infra::atomic_write_json(&summary_path, &summary)?;

    Ok(ToolInvocationResult {
        stage_result,
        runtime_provenance_path,
        stage_manifest_path,
        stdout_path,
        stderr_path,
        summary_path,
    })
}
