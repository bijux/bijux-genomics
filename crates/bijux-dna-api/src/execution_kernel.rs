use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{anyhow, bail, Context, Result};
use bijux_dna_core::contract::ExecutionStep;
use bijux_dna_environment::api::RuntimeKind;
use bijux_dna_runner::execute::{execute_step, StageResultV1};
use bijux_dna_runtime::recording::write_execution_logs_bounded;
use serde::{Deserialize, Serialize};

use crate::writers::ArtifactWriter;

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
    #[serde(default)]
    pub mode: ToolExecMode,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ToolExecMode {
    #[default]
    Execute,
    DryRun,
    PrintCommands,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ToolInvocationResult {
    pub stage_result: StageResultV1,
    pub runtime_provenance_path: PathBuf,
    pub stage_manifest_path: PathBuf,
    pub stdout_path: PathBuf,
    pub stderr_path: PathBuf,
    pub summary_path: PathBuf,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ToolExec;

impl ToolExec {
    /// # Errors
    /// Returns an error if path contracts fail, tool execution fails, or artifacts cannot be written.
    pub fn invoke(req: &ToolInvocationRequest) -> Result<ToolInvocationResult> {
        invoke_tool(req)
    }
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

fn require_pinned_digest(step: &ExecutionStep) -> Result<()> {
    let digest = step
        .image
        .digest
        .as_deref()
        .ok_or_else(|| anyhow!("tool resolution failed: missing image digest for {}", step.image.image))?;
    if !digest.starts_with("sha256:") || digest.len() < 16 {
        bail!(
            "tool resolution failed: unpinned/invalid digest `{digest}` for {}",
            step.image.image
        );
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

fn validate_required_outputs(step: &ExecutionStep) -> Result<()> {
    for artifact in &step.io.outputs {
        if artifact.optional {
            continue;
        }
        if !artifact.path.exists() {
            bail!(
                "stage contract violation: required output '{}' was not produced at {}",
                artifact.name,
                artifact.path.display()
            );
        }
    }
    Ok(())
}

fn output_checksums(step: &ExecutionStep) -> Result<serde_json::Value> {
    ArtifactWriter::write_output_checksums(&step.out_dir, &step.io.outputs)
}

fn can_resume(req: &ToolInvocationRequest) -> Result<bool> {
    let manifest_path = req.context.stage_root.join("stage_manifest.json");
    if !manifest_path.exists() {
        return Ok(false);
    }
    let all_outputs_exist = req
        .step
        .io
        .outputs
        .iter()
        .all(|artifact| artifact.optional || artifact.path.exists());
    if !all_outputs_exist {
        return Ok(false);
    }
    let raw = std::fs::read_to_string(&manifest_path)?;
    let manifest: serde_json::Value = serde_json::from_str(&raw)?;
    let stored = manifest.get("output_checksums").cloned().unwrap_or(serde_json::Value::Null);
    let current = output_checksums(&req.step)?;
    Ok(stored == current && stored != serde_json::Value::Null)
}

fn write_crash_bundle(
    req: &ToolInvocationRequest,
    stderr_tail: &str,
    exit_code: i32,
    command: &str,
) -> Result<PathBuf> {
    let crash_path = req.context.stage_root.join("crash.json");
    let mut inputs = serde_json::Map::new();
    for artifact in &req.step.io.inputs {
        if artifact.path.exists() {
            let checksum = bijux_dna_infra::hash_file_sha256(&artifact.path).unwrap_or_else(|_| "unknown".to_string());
            inputs.insert(
                artifact.name.to_string(),
                serde_json::json!({"path": artifact.path, "sha256": checksum}),
            );
        }
    }
    let env_subset = serde_json::json!({
        "LC_ALL": std::env::var("LC_ALL").ok(),
        "LANG": std::env::var("LANG").ok(),
        "TZ": std::env::var("TZ").ok(),
        "TMPDIR": std::env::var("TMPDIR").ok(),
        "HOME": std::env::var("HOME").ok(),
    });
    let payload = serde_json::json!({
        "schema_version": "bijux.stage_crash.v1",
        "run_id": req.context.run_id,
        "stage_id": req.context.stage_id,
        "tool_id": req.context.tool_id,
        "exit_code": exit_code,
        "command": command,
        "stderr_tail": stderr_tail,
        "inputs": inputs,
        "env_summary": env_subset,
        "tool_digest": req.step.image.digest,
    });
    bijux_dna_infra::atomic_write_json(&crash_path, &payload)?;
    Ok(crash_path)
}

/// # Errors
/// Returns an error if path contracts fail, tool execution fails, or artifacts cannot be written.
pub fn invoke_tool(req: &ToolInvocationRequest) -> Result<ToolInvocationResult> {
    enforce_path_contracts(req)?;
    require_pinned_digest(&req.step)?;
    crate::input_validation::validate_stage_inputs(&req.step)?;
    if network_policy_violation(&req.context.network_policy) {
        bail!(
            "network policy violation: {} forbids network but BIJUX_ALLOW_NETWORK is enabled",
            req.context.stage_id
        );
    }
    bijux_dna_infra::ensure_dir(&req.context.stage_root)?;
    bijux_dna_infra::ensure_dir(&req.context.tmp_root)?;
    let work_dir = req
        .context
        .output_root
        .join("work")
        .join(&req.context.run_id)
        .join(&req.context.stage_id)
        .join(req.step.step_id.as_str());
    bijux_dna_infra::ensure_dir(&work_dir)?;
    let home_dir = work_dir.join("home");
    bijux_dna_infra::ensure_dir(&home_dir)?;
    std::env::set_var("LC_ALL", "C");
    std::env::set_var("LANG", "C");
    std::env::set_var("TZ", "UTC");
    std::env::set_var("BIJUX_STAGE_THREADS", req.context.threads.to_string());
    if let Some(memory_hint_mb) = req.context.memory_hint_mb {
        std::env::set_var("BIJUX_STAGE_MEMORY_MB", memory_hint_mb.to_string());
    }
    if let Some(seed) = req.context.seed {
        std::env::set_var("BIJUX_STAGE_SEED", seed.to_string());
    }
    std::env::set_var("TMPDIR", &req.context.tmp_root);
    std::env::set_var("HOME", &home_dir);

    if req.mode == ToolExecMode::PrintCommands || req.mode == ToolExecMode::DryRun {
        let dry_path = req.context.stage_root.join("print_commands.txt");
        let image = req.step.image.image.clone();
        let digest = req.step.image.digest.clone().unwrap_or_default();
        let cmd = format!(
            "{} run {} {}",
            req.runner,
            image,
            req.step.command.template.join(" ")
        );
        bijux_dna_infra::atomic_write_bytes(&dry_path, cmd.as_bytes())?;
        if req.mode == ToolExecMode::DryRun {
            let summary_path = req.context.stage_root.join("stage_human_summary.json");
            let summary = serde_json::json!({
                "schema_version": "bijux.stage_summary.v1",
                "stage_id": req.context.stage_id,
                "tool_id": req.context.tool_id,
                "status": "dry_run",
                "command": cmd,
                "tool_digest": digest,
            });
            bijux_dna_infra::atomic_write_json(&summary_path, &summary)?;
        }
        return Ok(ToolInvocationResult {
            stage_result: StageResultV1 {
                run_id: req.context.run_id.clone(),
                exit_code: 0,
                runtime_s: 0.0,
                memory_mb: 0.0,
                outputs: vec![],
                metrics_path: None,
                stdout: String::new(),
                stderr: String::new(),
                command: cmd,
            },
            runtime_provenance_path: req.context.stage_root.join("runtime_provenance.json"),
            stage_manifest_path: req.context.stage_root.join("stage_manifest.json"),
            stdout_path: req.context.stage_root.join("logs").join("tool.stdout.log"),
            stderr_path: req.context.stage_root.join("logs").join("tool.stderr.log"),
            summary_path: req.context.stage_root.join("stage_human_summary.json"),
        });
    }
    if can_resume(req)? {
        return Ok(ToolInvocationResult {
            stage_result: StageResultV1 {
                run_id: req.context.run_id.clone(),
                exit_code: 0,
                runtime_s: 0.0,
                memory_mb: 0.0,
                outputs: req.step.io.outputs.iter().map(|a| a.path.clone()).collect(),
                metrics_path: None,
                stdout: String::new(),
                stderr: String::new(),
                command: "resume-skip".to_string(),
            },
            runtime_provenance_path: req.context.stage_root.join("runtime_provenance.json"),
            stage_manifest_path: req.context.stage_root.join("stage_manifest.json"),
            stdout_path: req.context.stage_root.join("logs").join("tool.stdout.log"),
            stderr_path: req.context.stage_root.join("logs").join("tool.stderr.log"),
            summary_path: req.context.stage_root.join("stage_human_summary.json"),
        });
    }

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
    validate_required_outputs(&req.step)?;
    let output_checksums = ArtifactWriter::write_output_checksums(
        &req.context.stage_root,
        &req.step.io.outputs,
    )
    .context("write stage artifact checksums")?;
    let log_paths = write_execution_logs_bounded(&logs_dir, &stage_result.stdout, &stage_result.stderr)
        .context("write stage logs")?;
    let stdout_path = logs_dir.join("tool.stdout.log");
    let stderr_path = logs_dir.join("tool.stderr.log");
    let stderr_tail = std::fs::read_to_string(&stderr_path).unwrap_or_default();
    let stdout_tail = std::fs::read_to_string(&stdout_path).unwrap_or_default();

    let exit_taxonomy = classify_exit_code(stage_result.exit_code);
    if stage_result.exit_code != 0 {
        let _ = write_crash_bundle(req, &stderr_tail, stage_result.exit_code, &stage_result.command);
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
        "output_checksums": output_checksums,
        "runtime": {
            "runtime_s": stage_result.runtime_s,
            "memory_mb": stage_result.memory_mb,
            "exit_code": stage_result.exit_code,
        },
        "logs": log_paths,
        "runtime_provenance": runtime_provenance_path,
    });
    ArtifactWriter::write_stage_manifest(&stage_manifest_path, &stage_manifest)?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use bijux_dna_core::contract::{ArtifactRole, ArtifactSpec, StageIO, ToolConstraints};
    use bijux_dna_core::prelude::{CommandSpecV1, ContainerImageRefV1};
    use bijux_dna_core::ids::{ArtifactId, StageId, StepId};
    use tempfile::TempDir;

    #[test]
    fn tool_exec_bcftools_version_in_container() -> Result<()> {
        if std::env::var("BIJUX_E2E").is_err() {
            return Ok(());
        }
        let tmp = TempDir::new()?;
        let stage_root = tmp.path().join("artifacts").join("stage");
        let out_root = tmp.path().join("out");
        let in_root = tmp.path().join("in");
        bijux_dna_infra::ensure_dir(&stage_root)?;
        bijux_dna_infra::ensure_dir(&out_root)?;
        bijux_dna_infra::ensure_dir(&in_root)?;
        let out_file = out_root.join("bcftools.version.txt");
        let step = ExecutionStep {
            step_id: StepId::new("vcf.stats.bcftools_version"),
            stage_id: StageId::new("vcf.stats"),
            command: CommandSpecV1 {
                template: vec![
                    "sh".to_string(),
                    "-lc".to_string(),
                    "bcftools --version > /data/output/bcftools.version.txt".to_string(),
                ],
            },
            image: ContainerImageRefV1 {
                image: "quay.io/biocontainers/bcftools:1.20--h8b25389_0".to_string(),
                digest: Some(
                    "sha256:67f54df47f501f6ddef08e3b9ad89cf693952f9a89de0d74df6e39fce15f1ff6"
                        .to_string(),
                ),
            },
            resources: ToolConstraints::default(),
            io: StageIO {
                inputs: vec![],
                outputs: vec![ArtifactSpec::required(
                    ArtifactId::new("version"),
                    out_file.clone(),
                    ArtifactRole::Log,
                )],
            },
            out_dir: out_root.clone(),
            aux_images: std::collections::BTreeMap::new(),
            expected_artifact_ids: vec![],
            metrics_schema_ids: vec![],
        };
        let req = ToolInvocationRequest {
            step,
            runner: RuntimeKind::Docker,
            context: ToolContext {
                run_id: "run-e2e-tool-exec".to_string(),
                stage_id: "vcf.stats".to_string(),
                tool_id: "bcftools".to_string(),
                sample_id: None,
                stage_root: stage_root.clone(),
                input_root: in_root,
                output_root: out_root.clone(),
                tmp_root: stage_root.join("tmp"),
                threads: 1,
                memory_hint_mb: Some(512),
                seed: Some(7),
                network_policy: NetworkPolicy::Forbid,
            },
            timeout: None,
            mode: ToolExecMode::Execute,
        };
        let result = ToolExec::invoke(&req)?;
        assert_eq!(result.stage_result.exit_code, 0);
        let version = std::fs::read_to_string(out_file)?;
        assert!(
            version.to_ascii_lowercase().contains("bcftools"),
            "unexpected bcftools --version output: {version}"
        );
        Ok(())
    }
}
