use std::path::PathBuf;
use std::time::Duration;

use anyhow::{anyhow, bail, Context, Result};
use bijux_dna_core::contract::ExecutionStep;
use bijux_dna_environment::api::RuntimeKind;
use bijux_dna_runner::execute::{execute_step, StageResultV1};
use bijux_dna_runtime::recording::write_execution_logs_bounded;
use serde::{Deserialize, Serialize};

use crate::writers::{ArtifactWriter, MetricsWriter};
#[path = "execution_kernel_support.rs"]
mod execution_kernel_support;
use execution_kernel_support::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum NetworkPolicy {
    Allow,
    Forbid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ToolContext {
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
    pub compression_threads: Option<u32>,
    pub seed: Option<u64>,
    pub network_policy: NetworkPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ToolInvocationRequest {
    pub step: ExecutionStep,
    pub runner: RuntimeKind,
    pub context: ToolContext,
    pub timeout: Option<Duration>,
    #[serde(default)]
    pub mode: ToolExecMode,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ToolExecMode {
    #[default]
    Execute,
    DryRun,
    DryRunExplain,
    PrintCommands,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct ToolInvocationResult {
    pub stage_result: StageResultV1,
    pub runtime_provenance_path: PathBuf,
    pub stage_manifest_path: PathBuf,
    pub stdout_path: PathBuf,
    pub stderr_path: PathBuf,
    pub summary_path: PathBuf,
}

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct ToolExec;

impl ToolExec {
    /// # Errors
    /// Returns an error if path contracts fail, tool execution fails, or artifacts cannot be written.
    pub fn invoke(req: &ToolInvocationRequest) -> Result<ToolInvocationResult> {
        invoke_tool(req)
    }
}

/// # Errors
/// Returns an error if path contracts fail, tool execution fails, or artifacts cannot be written.
pub fn invoke_tool(req: &ToolInvocationRequest) -> Result<ToolInvocationResult> {
    enforce_path_contracts(req)?;
    require_pinned_digest(&req.step)?;
    crate::input_validation::validate_stage_inputs(&req.step)?;
    enforce_seed_policy(req)?;
    let policy = effective_runtime_policy(req);
    if network_policy_violation(&req.context.network_policy) {
        let _ = bijux_dna_infra::atomic_write_json(
            &req.context.stage_root.join("stage_result_status.json"),
            &serde_json::json!({
                "schema_version": "bijux.stage_result_status.v1",
                "status": "refused",
                "reason_code": "NETWORK_FORBIDDEN",
            }),
        );
        let _ = bijux_dna_runtime::recording::write_run_artifact_envelope(
            &req.context.stage_root,
            &req.context.stage_id,
            bijux_dna_runtime::recording::StageResultStatus::Refused,
            "NETWORK_FORBIDDEN",
        );
        bail!(
            "network policy violation: {} forbids network but BIJUX_ALLOW_NETWORK is enabled",
            req.context.stage_id
        );
    }
    bijux_dna_infra::ensure_dir(&req.context.stage_root)?;
    let effective_tmp_root = policy
        .temp_root
        .clone()
        .unwrap_or_else(|| req.context.tmp_root.clone());
    bijux_dna_infra::ensure_dir(&effective_tmp_root)?;
    let work_dir = req
        .context
        .output_root
        .join("work")
        .join(&req.context.run_id)
        .join(&req.context.stage_id)
        .join(req.step.step_id.as_str());
    bijux_dna_infra::ensure_dir(&work_dir)?;
    let cache_root = policy
        .cache_root
        .clone()
        .unwrap_or_else(|| req.context.output_root.join("cache"));
    bijux_dna_infra::ensure_dir(&cache_root)?;
    let home_dir = work_dir.join("home");
    bijux_dna_infra::ensure_dir(&home_dir)?;
    std::env::set_var(
        "LC_ALL",
        policy.deterministic_env.lc_all.as_deref().unwrap_or("C"),
    );
    std::env::set_var(
        "LANG",
        policy.deterministic_env.lang.as_deref().unwrap_or("C"),
    );
    std::env::set_var(
        "TZ",
        policy.deterministic_env.tz.as_deref().unwrap_or("UTC"),
    );
    if let Some(path) = policy.deterministic_env.path.as_deref() {
        std::env::set_var("PATH", path);
    }
    if let Some(umask) = policy.deterministic_env.umask.as_deref() {
        std::env::set_var("BIJUX_UMASK", umask);
    }
    std::env::set_var("BIJUX_STAGE_THREADS", policy.threads.to_string());
    if let Some(memory_hint_mb) = policy.memory_mb {
        std::env::set_var("BIJUX_STAGE_MEMORY_MB", memory_hint_mb.to_string());
    }
    if let Some(compression_threads) = policy.compression_threads {
        std::env::set_var("BIJUX_COMPRESSION_THREADS", compression_threads.to_string());
    }
    if let Some(seed) = req.context.seed {
        std::env::set_var("BIJUX_STAGE_SEED", seed.to_string());
    }
    std::env::set_var("TMPDIR", &effective_tmp_root);
    std::env::set_var("HOME", &home_dir);
    std::env::set_var("XDG_CACHE_HOME", &cache_root);
    std::env::set_var("BIJUX_CACHE_ROOT", &cache_root);
    for var in ["XDG_CACHE_HOME", "BIJUX_CACHE_ROOT"] {
        if let Ok(value) = std::env::var(var) {
            let path = PathBuf::from(value);
            if !path.starts_with(&cache_root) {
                bail!(
                    "cache policy violation: {var} must be under {}",
                    cache_root.display()
                );
            }
        }
    }

    let lock_root = req.context.output_root.join(".runtime_locks");
    bijux_dna_infra::ensure_dir(&lock_root)?;
    let is_heavy = policy
        .heavy_patterns
        .iter()
        .any(|pattern| stage_matches(pattern, &req.context.stage_id));
    let _heavy_lock = if is_heavy {
        acquire_slot_lock(&lock_root, "heavy", policy.max_local_heavy_parallel)?
    } else {
        None
    };
    let command_line = req.step.command.template.join(" ").to_ascii_lowercase();
    let _io_lock = if command_line.contains("bgzip") || command_line.contains("tabix") {
        acquire_slot_lock(&lock_root, "bgzip_tabix", policy.bgzip_tabix_max_parallel)?
    } else {
        None
    };

    if req.mode == ToolExecMode::PrintCommands
        || req.mode == ToolExecMode::DryRun
        || req.mode == ToolExecMode::DryRunExplain
    {
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
        if req.mode == ToolExecMode::DryRun || req.mode == ToolExecMode::DryRunExplain {
            let summary_path = req.context.stage_root.join("stage_human_summary.json");
            let stage_status_path = req.context.stage_root.join("stage_result_status.json");
            let summary = serde_json::json!({
                "schema_version": "bijux.stage_summary.v1",
                "stage_id": req.context.stage_id,
                "tool_id": req.context.tool_id,
                "status": "dry_run",
                "command": cmd,
                "tool_digest": digest,
            });
            bijux_dna_infra::atomic_write_json(&summary_path, &summary)?;
            bijux_dna_infra::atomic_write_json(
                &stage_status_path,
                &serde_json::json!({
                    "schema_version": "bijux.stage_result_status.v1",
                    "status": "ok",
                    "reason_code": "DRY_RUN"
                }),
            )?;
            let _ = bijux_dna_runtime::recording::write_run_artifact_envelope(
                &req.context.stage_root,
                &req.context.stage_id,
                bijux_dna_runtime::recording::StageResultStatus::Ok,
                "DRY_RUN",
            );
            if req.mode == ToolExecMode::DryRunExplain {
                let explain_path = req.context.stage_root.join("dry_run_explain.json");
                let payload = serde_json::json!({
                    "schema_version": "bijux.dry_run_explain.v1",
                    "stage_id": req.context.stage_id,
                    "tool_id": req.context.tool_id,
                    "runner": req.runner,
                    "command": req.step.command.template,
                    "io": req.step.io,
                    "resources": {
                        "threads": policy.threads,
                        "memory_mb": policy.memory_mb,
                        "compression_threads": policy.compression_threads,
                        "timeout_s": policy.timeout.map(|d| d.as_secs()),
                        "temp_root": effective_tmp_root,
                        "cache_root": cache_root,
                    },
                });
                bijux_dna_infra::atomic_write_json(&explain_path, &payload)?;
            }
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
        let _ = update_resume_report(
            &req.context.stage_root,
            &req.context.stage_id,
            "cached",
            "CACHE_HIT",
        );
        let stage_status_path = req.context.stage_root.join("stage_result_status.json");
        bijux_dna_infra::atomic_write_json(
            &stage_status_path,
            &serde_json::json!({
                "schema_version": "bijux.stage_result_status.v1",
                "status": "skipped_cached",
                "reason_code": "CACHE_HIT"
            }),
        )?;
        let _ = bijux_dna_runtime::recording::write_run_artifact_envelope(
            &req.context.stage_root,
            &req.context.stage_id,
            bijux_dna_runtime::recording::StageResultStatus::SkippedCached,
            "CACHE_HIT",
        );
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
    let _ = update_resume_report(
        &req.context.stage_root,
        &req.context.stage_id,
        "recomputed",
        "CACHE_MISS_OR_DISABLED",
    );

    let logs_dir = req.context.stage_root.join("logs");
    bijux_dna_infra::ensure_dir(&logs_dir)?;
    let started_at = chrono::Utc::now();
    let stage_log_path = req.context.stage_root.join("stage_events.jsonl");
    let start_event = serde_json::json!({
        "schema_version": "bijux.stage_events.v1",
        "event": "stage_start",
        "run_id": req.context.run_id,
        "stage_id": req.context.stage_id,
        "tool_id": req.context.tool_id,
        "chunk_id": "global",
        "timestamp": started_at,
    });
    bijux_dna_runtime::recording::append_jsonl_line(
        &stage_log_path,
        &serde_json::to_string(&start_event)?,
    )?;
    let chunk_start_event = serde_json::json!({
        "schema_version": "bijux.stage_events.v1",
        "event": "chunk_start",
        "run_id": req.context.run_id,
        "stage_id": req.context.stage_id,
        "tool_id": req.context.tool_id,
        "chunk_id": "global",
        "timestamp": started_at,
    });
    bijux_dna_runtime::recording::append_jsonl_line(
        &stage_log_path,
        &serde_json::to_string(&chunk_start_event)?,
    )?;
    let tool_event = serde_json::json!({
        "schema_version": "bijux.stage_events.v1",
        "event": "tool_invoked",
        "run_id": req.context.run_id,
        "stage_id": req.context.stage_id,
        "tool_id": req.context.tool_id,
        "timestamp": started_at,
        "chunk_id": "global",
    });
    bijux_dna_runtime::recording::append_jsonl_line(
        &stage_log_path,
        &serde_json::to_string(&tool_event)?,
    )?;

    let arg_threshold = std::env::var("BIJUX_MAX_COMMAND_LENGTH")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(12_000);
    let effective_step = rewrite_long_region_args(&req.step, &work_dir, arg_threshold)?;
    let stage_result = execute_step(&effective_step, req.runner, policy.timeout)?;
    validate_required_outputs(&effective_step)?;
    enforce_large_file_guard(&req.context.output_root, &effective_step.io.outputs)?;
    let stage_metrics_path = req.context.stage_root.join("stage.metrics.json");
    let log_paths =
        write_execution_logs_bounded(&logs_dir, &stage_result.stdout, &stage_result.stderr)
            .context("write stage logs")?;
    let stdout_path = logs_dir.join("tool.stdout.log");
    let stderr_path = logs_dir.join("tool.stderr.log");
    let stderr_tail = std::fs::read_to_string(&stderr_path).unwrap_or_default();
    let stdout_tail = std::fs::read_to_string(&stdout_path).unwrap_or_default();

    let exit_taxonomy = classify_exit_code(stage_result.exit_code);
    if stage_result.exit_code != 0 {
        let _ = write_crash_bundle(
            req,
            &stderr_tail,
            stage_result.exit_code,
            &stage_result.command,
        );
        let _ = mark_partial_failure_invalid(&req.context.stage_root, &effective_step.io.outputs);
        let _ = bijux_dna_infra::atomic_write_json(
            &req.context.stage_root.join("stage_result_status.json"),
            &serde_json::json!({
                "schema_version": "bijux.stage_result_status.v1",
                "status": "failed",
                "reason_code": format!("{exit_taxonomy:?}"),
            }),
        );
        let _ = bijux_dna_runtime::recording::write_run_artifact_envelope(
            &req.context.stage_root,
            &req.context.stage_id,
            bijux_dna_runtime::recording::StageResultStatus::Failed,
            &format!("{exit_taxonomy:?}"),
        );
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
    let duration_ms = finished_at
        .signed_duration_since(started_at)
        .num_milliseconds()
        .max(0);
    let inferred_tool_version = infer_tool_version_from_image(&req.step.image.image);
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
        "tool_version": inferred_tool_version,
        "tool_version_probe_cmd": format!("{} --version", req.context.tool_id),
        "tool_version_probe_output": infer_version_line(&stage_result.stdout, &stage_result.stderr),
        "command": stage_result.command,
        "env_summary": env_summary,
        "started_at": started_at,
        "finished_at": finished_at,
        "duration_ms": duration_ms,
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
        "threads": policy.threads,
        "memory_hint_mb": policy.memory_mb,
        "compression_threads": policy.compression_threads,
        "seed": req.context.seed,
        "network_policy": req.context.network_policy,
        "inputs": effective_step.io.inputs,
        "outputs": effective_step.io.outputs,
        "runtime": {
            "runtime_s": stage_result.runtime_s,
            "duration_ms": duration_ms,
            "memory_mb": stage_result.memory_mb,
            "exit_code": stage_result.exit_code,
        },
        "logs": log_paths,
        "runtime_provenance": runtime_provenance_path,
    });
    let (output_checksums, _manifest_path) = ArtifactWriter::write_stage_outputs_and_manifest(
        &req.context.stage_root,
        &effective_step.io.outputs,
        &stage_manifest_path,
        stage_manifest,
    )
    .context("write stage artifact checksums + stage manifest")?;

    let mut stage_metrics = serde_json::json!({
        "schema_version": "bijux.stage.metrics.v1",
        "stage_id": req.context.stage_id,
        "tool_id": req.context.tool_id,
        "runtime_s": stage_result.runtime_s,
        "wall_time_ms": duration_ms,
        "memory_mb": stage_result.memory_mb,
        "exit_code": stage_result.exit_code,
        "threads": policy.threads,
        "output_checksums": output_checksums,
    });
    if req.context.stage_id.starts_with("vcf.") {
        stage_metrics["records_in"] = serde_json::json!(0);
        stage_metrics["records_out"] = serde_json::json!(0);
    }
    if req.context.stage_id.starts_with("fastq.") {
        stage_metrics["reads_in"] = serde_json::json!(0);
        stage_metrics["reads_out"] = serde_json::json!(0);
    }
    MetricsWriter::write_stage_metrics(&stage_metrics_path, &req.context.stage_id, &stage_metrics)
        .context("write stage metrics with schema-backed required keys")?;

    let end_event = serde_json::json!({
        "schema_version": "bijux.stage_events.v1",
        "event": "chunk_end",
        "run_id": req.context.run_id,
        "stage_id": req.context.stage_id,
        "tool_id": req.context.tool_id,
        "chunk_id": "global",
        "timestamp": finished_at,
        "runtime_s": stage_result.runtime_s,
        "exit_code": stage_result.exit_code,
    });
    bijux_dna_runtime::recording::append_jsonl_line(
        &stage_log_path,
        &serde_json::to_string(&end_event)?,
    )?;
    let stage_end_event = serde_json::json!({
        "schema_version": "bijux.stage_events.v1",
        "event": "stage_end",
        "run_id": req.context.run_id,
        "stage_id": req.context.stage_id,
        "tool_id": req.context.tool_id,
        "chunk_id": "global",
        "timestamp": finished_at,
        "runtime_s": stage_result.runtime_s,
        "exit_code": stage_result.exit_code,
    });
    bijux_dna_runtime::recording::append_jsonl_line(
        &stage_log_path,
        &serde_json::to_string(&stage_end_event)?,
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
    bijux_dna_infra::atomic_write_json(
        &req.context.stage_root.join("stage_result_status.json"),
        &serde_json::json!({
            "schema_version": "bijux.stage_result_status.v1",
            "status": "ok",
            "reason_code": "SUCCESS",
        }),
    )?;
    let _ = bijux_dna_runtime::recording::write_run_artifact_envelope(
        &req.context.stage_root,
        &req.context.stage_id,
        bijux_dna_runtime::recording::StageResultStatus::Ok,
        "SUCCESS",
    );

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
mod execution_kernel_tests;
