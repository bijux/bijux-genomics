use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::anyhow;
use bijux_dna_core::contract::ExecutionStep;

use super::{Result, ToolInvocationRequest};
use crate::runtime::persistence::ArtifactWriter;

pub(crate) fn acquire_slot_lock(
    base: &Path,
    prefix: &str,
    slots: u32,
) -> Result<Option<bijux_dna_infra::FileLock>> {
    if slots <= 1 {
        let lock = bijux_dna_infra::FileLock::acquire(
            &base.join(format!("{prefix}.lock")),
            Duration::from_secs(300),
        )
        .map_err(|err| anyhow!("acquire {prefix} lock: {err}"))?;
        return Ok(Some(lock));
    }
    for slot in 0..slots {
        let path = base.join(format!("{prefix}.{slot}.lock"));
        if let Ok(lock) = bijux_dna_infra::FileLock::acquire(&path, Duration::from_millis(150)) {
            return Ok(Some(lock));
        }
    }
    let lock = bijux_dna_infra::FileLock::acquire(
        &base.join(format!("{prefix}.0.lock")),
        Duration::from_secs(300),
    )
    .map_err(|err| anyhow!("acquire {prefix} lock: {err}"))?;
    Ok(Some(lock))
}

pub(super) fn output_checksums(step: &ExecutionStep) -> Result<serde_json::Value> {
    ArtifactWriter::write_output_checksums(&step.out_dir, &step.io.outputs)
}

pub(crate) fn can_resume(req: &ToolInvocationRequest) -> Result<bool> {
    let manifest_path = req.context.stage_root.join("stage_manifest.json");
    if !manifest_path.exists() {
        return Ok(false);
    }
    let all_outputs_exist =
        req.step.io.outputs.iter().all(|artifact| artifact.optional || artifact.path.exists());
    if !all_outputs_exist {
        return Ok(false);
    }
    let raw = std::fs::read_to_string(&manifest_path)?;
    let manifest: serde_json::Value = serde_json::from_str(&raw)?;
    let stored = manifest.get("output_checksums").cloned().unwrap_or(serde_json::Value::Null);
    let current = output_checksums(&req.step)?;
    Ok(stored == current && stored != serde_json::Value::Null)
}

pub(crate) fn update_resume_report(
    stage_root: &Path,
    stage_id: &str,
    status: &str,
    reason: &str,
) -> Result<()> {
    let report_path = stage_root.join("resume_report.json");
    let mut cached = Vec::<serde_json::Value>::new();
    let mut recomputed = Vec::<serde_json::Value>::new();
    if report_path.exists() {
        let existing: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&report_path)?)?;
        cached = existing
            .get("cached")
            .and_then(serde_json::Value::as_array)
            .cloned()
            .ok_or_else(|| anyhow!("resume report missing declared `cached` array"))?;
        recomputed = existing
            .get("recomputed")
            .and_then(serde_json::Value::as_array)
            .cloned()
            .ok_or_else(|| anyhow!("resume report missing declared `recomputed` array"))?;
    }
    let entry = serde_json::json!({
        "stage_id": stage_id,
        "reason": reason,
        "timestamp": chrono::Utc::now(),
    });
    if status == "cached" {
        cached.push(entry);
    } else {
        recomputed.push(entry);
    }
    let payload = serde_json::json!({
        "schema_version": "bijux.resume_report.v1",
        "cached": cached,
        "recomputed": recomputed,
        "summary": {
            "cached_count": cached.len(),
            "recomputed_count": recomputed.len(),
        }
    });
    bijux_dna_infra::atomic_write_json(&report_path, &payload)?;
    Ok(())
}

pub(crate) fn mark_partial_failure_invalid(
    stage_root: &Path,
    outputs: &[bijux_dna_core::contract::ArtifactSpec],
) -> Result<()> {
    let invalid_dir = stage_root.join("invalid_artifacts");
    bijux_dna_infra::ensure_dir(&invalid_dir)?;
    let mut invalid = Vec::<serde_json::Value>::new();
    for artifact in outputs {
        if artifact.path.exists() {
            let marker = invalid_dir.join(format!("{}.invalid", artifact.name));
            bijux_dna_infra::atomic_write_bytes(
                &marker,
                format!("invalidated_after_failure={}\n", chrono::Utc::now()).as_bytes(),
            )?;
            invalid.push(serde_json::json!({
                "name": artifact.name,
                "path": artifact.path,
                "marker": marker,
                "status": "invalid_after_failure",
            }));
        }
    }
    let payload = serde_json::json!({
        "schema_version": "bijux.partial_failure_cleanup.v1",
        "policy": "keep_intermediates_mark_invalid",
        "items": invalid,
    });
    bijux_dna_infra::atomic_write_json(&stage_root.join("partial_failure_cleanup.json"), &payload)?;
    Ok(())
}

pub(crate) fn write_crash_bundle(
    req: &ToolInvocationRequest,
    stderr_tail: Option<&str>,
    exit_code: i32,
    command: &str,
) -> Result<PathBuf> {
    let crash_path = req.context.stage_root.join("crash_provenance.json");
    let mut input_list = Vec::new();
    for artifact in &req.step.io.inputs {
        if artifact.path.exists() {
            let checksum = bijux_dna_infra::hash_file_sha256(&artifact.path).ok();
            input_list.push(serde_json::json!({
                "name": artifact.name,
                "path": artifact.path,
                "sha256": checksum
            }));
        }
    }
    let stderr_last_lines = stderr_tail
        .map(|tail| {
            tail.lines()
                .rev()
                .take(50)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .map(std::string::ToString::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
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
        "stderr_last_lines": stderr_last_lines,
        "inputs": input_list,
        "env_summary": env_subset,
        "tool_digest": req.step.image.digest,
    });
    bijux_dna_infra::atomic_write_json(&crash_path, &payload)?;
    Ok(crash_path)
}
