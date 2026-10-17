//! An interrupted recorded rename must remain resumable under its original byte authority.
use std::path::Path;

use anyhow::{ensure, Context as _, Result};
use bijux_dna_containers::api::{
    apply_result_layout, derive_result_layout_plan, rollback_result_layout, verify_result_layout,
    ResultLayoutPlanRequest,
};

pub(super) fn require_recorded_rename_recovery(runs: &Path, result_root: &Path) -> Result<()> {
    let plan = derive_result_layout_plan(&ResultLayoutPlanRequest {
        runs_root: runs.to_path_buf(),
        result_root: result_root.to_path_buf(),
        created_at_utc: "2026-08-07T03:03:00Z".to_string(),
    })?;
    ensure!(plan.moves.len() == 1, "existing governed fixture must select its actual move");
    let movement = plan.moves.first().context("actual recorded move")?;
    let source = result_root.join(&movement.source);
    let target = result_root.join(&movement.target);
    ensure!(source.is_dir() && !target.exists(), "recorded rename control");
    let source_bytes = std::fs::read(source.join("sample.bam"))?;
    let source_permissions = std::fs::metadata(source.join("sample.bam"))?.permissions();
    let plan_path = result_root.join(".governance/interrupted-result-layout.json");
    bijux_dna_infra::atomic_write_json(&plan_path, &plan)?;
    let original_plan = std::fs::read(&plan_path)?;
    // This is the real effect boundary between an atomic rename and progress publication.
    std::fs::rename(&source, &target)?;
    ensure!(
        !source.exists() && target.is_dir() && std::fs::read(&plan_path)? == original_plan,
        "interruption must retain the unmarked original plan and actual renamed bytes"
    );
    eprintln!("RESULT_LAYOUT_INTERRUPTION_CONTROL_PASS: cold apply/verify/rollback and recorded unmarked rename");
    let resumed = apply_result_layout(&plan_path, runs, result_root, "2026-08-07T03:04:00Z");
    let verified = if resumed.is_ok() {
        Some(verify_result_layout(&plan_path, runs, result_root))
    } else {
        None
    };
    if resumed.is_ok() {
        rollback_result_layout(&plan_path, runs, result_root, "2026-08-07T03:05:00Z")?;
    } else {
        ensure!(!source.exists() && target.is_dir(), "refusal must preserve interrupted bytes");
        std::fs::create_dir_all(source.parent().context("original result parent")?)?;
        std::fs::rename(&target, &source)?;
    }
    std::fs::write(&plan_path, &original_plan)?;
    ensure!(
        std::fs::read(source.join("sample.bam"))? == source_bytes
            && std::fs::metadata(source.join("sample.bam"))?.permissions() == source_permissions
            && std::fs::read(&plan_path)? == original_plan,
        "fixture source bytes/mode and original plan must restore before causal comparison"
    );
    eprintln!("RESULT_LAYOUT_INTERRUPTION_OBSERVED: resume={resumed:?}; verification={verified:?}");
    let resumed =
        resumed.context("recorded completed rename must resume under unchanged evidence")?;
    ensure!(resumed == plan.moves.len(), "resume must return actual recorded movement count");
    let verified = verified.context("successful resume must be verifiable")??;
    ensure!(verified == plan.moves.len(), "verification must retain the recorded movement count");
    Ok(())
}

#[test]
fn interrupted_result_move_refuses_unproven_target_without_recording_progress() -> Result<()> {
    for damage in ["changed", "extra", "missing", "both_present"] {
        let scratch = tempfile::tempdir()?;
        let (runs, result_root) = super::nested_result_fixture(scratch.path())?;
        let plan = derive_result_layout_plan(&ResultLayoutPlanRequest {
            runs_root: runs.clone(),
            result_root: result_root.clone(),
            created_at_utc: "2026-08-07T03:03:00Z".to_string(),
        })?;
        let movement = plan.moves.first().context("recorded move")?;
        let source = result_root.join(&movement.source);
        let target = result_root.join(&movement.target);
        let plan_path = scratch.path().join("interrupted.json");
        bijux_dna_infra::atomic_write_json(&plan_path, &plan)?;
        let original = std::fs::read(&plan_path)?;
        std::fs::rename(&source, &target)?;
        match damage {
            "changed" => std::fs::write(target.join("sample.bam"), b"changed-bam!")?,
            "extra" => std::fs::write(target.join("unrecorded.log"), b"foreign")?,
            "missing" => std::fs::remove_dir_all(&target)?,
            "both_present" => {
                std::fs::create_dir_all(&source)?;
                std::fs::write(source.join("sample.bam"), b"governed-bam")?;
            }
            _ => unreachable!(),
        }
        ensure!(
            apply_result_layout(&plan_path, &runs, &result_root, "2026-08-07T03:04:00Z").is_err(),
            "unproven {damage} target must refuse"
        );
        if damage == "both_present" {
            ensure!(
                source.is_dir() && target.is_dir(),
                "conflicting destinations must remain intact"
            );
        } else {
            ensure!(
                std::fs::read(&plan_path)? == original,
                "{damage} must refuse before recording progress"
            );
        }
    }
    Ok(())
}
