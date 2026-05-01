use super::Result;
use anyhow::{anyhow, Context};
use bijux_dna_runtime::run_layout::{
    RunCheckpointV1, RunFailureV1, RunLifecycleStateV1, RunStateV1,
};
use std::path::{Path, PathBuf};

/// Assess whether a failed run can be safely replayed.
///
/// # Errors
/// Returns an error if required run contracts cannot be loaded.
pub fn assess_failed_replay_eligibility(run_dir: &Path) -> Result<serde_json::Value> {
    let layout = bijux_dna_runtime::run_layout::RunLayout::from_run_dir(run_dir.to_path_buf());
    let run_state: RunStateV1 = serde_json::from_slice(&std::fs::read(&layout.run_state_path)?)
        .context("parse run state")?;
    let checkpoint: RunCheckpointV1 =
        serde_json::from_slice(&std::fs::read(&layout.checkpoint_path)?)
            .context("parse checkpoint")?;
    let failure: Option<RunFailureV1> = layout
        .failure_path
        .exists()
        .then(|| std::fs::read(&layout.failure_path).ok())
        .flatten()
        .and_then(|raw| serde_json::from_slice::<RunFailureV1>(&raw).ok());

    let failed =
        matches!(run_state.state, RunLifecycleStateV1::Failed | RunLifecycleStateV1::Cancelled);
    let reason_code = failure.as_ref().map(|record| record.failure_code.clone());
    let unsafe_resume = reason_code
        .as_deref()
        .is_some_and(|code| matches!(code, "invariant_violation" | "artifact_corruption"));
    let mut eligible_stage_ids = checkpoint.pending_stage_ids.clone();
    if let Some(stage_id) = failure.as_ref().and_then(|record| record.stage_id.clone()) {
        if !eligible_stage_ids.iter().any(|candidate| candidate == &stage_id) {
            eligible_stage_ids.insert(0, stage_id);
        }
    }
    Ok(serde_json::json!({
        "schema_version": "bijux.replay_failed_eligibility.v1",
        "run_id": run_state.run_id,
        "state": run_state.state,
        "failed": failed,
        "unsafe_resume": unsafe_resume,
        "failure_code": reason_code,
        "eligible_stage_ids": eligible_stage_ids,
        "refusal_reason": unsafe_resume.then_some("unsafe_resume_refused_due_to_failure_class"),
    }))
}

/// Replay a failed run into a sibling directory when resume safety checks pass.
///
/// # Errors
/// Returns an error if replay is unsafe or replay execution fails.
pub fn replay_failed_run(run_dir: &Path) -> Result<serde_json::Value> {
    let eligibility = assess_failed_replay_eligibility(run_dir)?;
    if eligibility.get("unsafe_resume").and_then(serde_json::Value::as_bool).unwrap_or(false) {
        return Err(anyhow!("unsafe resume refused for {}", run_dir.display()));
    }
    if !eligibility.get("failed").and_then(serde_json::Value::as_bool).unwrap_or(false) {
        return Err(anyhow!("run is not in a failed state: {}", run_dir.display()));
    }
    let replay_run_dir = next_replay_run_dir(run_dir);
    copy_tree(run_dir, &replay_run_dir)?;
    let manifest_path = replay_run_dir.join("run_manifest.json");
    super::replay_manifest(&manifest_path, false)?;
    Ok(serde_json::json!({
        "schema_version": "bijux.replay_failed_run.v1",
        "source_run_dir": run_dir.display().to_string(),
        "replay_run_dir": replay_run_dir.display().to_string(),
        "eligible_stage_ids": eligibility.get("eligible_stage_ids").cloned().unwrap_or(serde_json::json!([])),
    }))
}

fn next_replay_run_dir(run_dir: &Path) -> PathBuf {
    let base = run_dir
        .file_name()
        .and_then(|value| value.to_str())
        .map_or_else(|| "run".to_string(), ToString::to_string);
    let parent = run_dir.parent().unwrap_or(run_dir);
    for suffix in 1..=100 {
        let candidate = parent.join(format!("{base}.replay{suffix}"));
        if !candidate.exists() {
            return candidate;
        }
    }
    parent.join(format!("{base}.replay-final"))
}

fn copy_tree(src: &Path, dst: &Path) -> Result<()> {
    if !src.is_dir() {
        return Err(anyhow!("source run directory missing: {}", src.display()));
    }
    bijux_dna_infra::ensure_dir(dst)?;
    let mut stack = vec![(src.to_path_buf(), dst.to_path_buf())];
    while let Some((current_src, current_dst)) = stack.pop() {
        for entry in std::fs::read_dir(&current_src)? {
            let entry = entry?;
            let src_path = entry.path();
            let dst_path = current_dst.join(entry.file_name());
            if src_path.is_dir() {
                bijux_dna_infra::ensure_dir(&dst_path)?;
                stack.push((src_path, dst_path));
                continue;
            }
            std::fs::copy(&src_path, &dst_path).with_context(|| {
                format!("copy {} -> {}", src_path.display(), dst_path.display())
            })?;
        }
    }
    Ok(())
}
