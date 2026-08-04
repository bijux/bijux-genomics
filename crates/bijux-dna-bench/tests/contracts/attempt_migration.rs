use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use bijux_dna_bench::{
    apply_attempt_migration, plan_attempt_migration, rollback_attempt_migration,
    verify_attempt_migration, write_attempt_migration_plan, BenchmarkAttemptDisposition,
    BenchmarkAttemptMigrationState,
};
use serde_json::json;

fn write(path: &Path, contents: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, contents)?;
    Ok(())
}

fn file_record(root: &Path, relative: &str) -> Result<serde_json::Value> {
    let path = root.join(relative);
    Ok(json!({
        "path": relative,
        "size_bytes": path.metadata()?.len(),
        "sha256": bijux_dna_infra::hash_file_sha256(&path)?,
    }))
}

fn historical_attempt(
    study: &Path,
    scope: &str,
    run_id: &str,
    tool_id: &str,
    job_id: &str,
) -> Result<PathBuf> {
    let root = study
        .join("containers/evidence")
        .join(scope)
        .join(format!("{run_id}-{tool_id}-slurm-{job_id}"));
    let pipeline = format!(
        "schema_version: bijux.container.pipeline.v2\nrun:\n  id: {run_id}\n  kind: benchmark\n  tool: {tool_id}\n"
    );
    write(&root.join("submission/config-pipeline.yml"), &pipeline)?;
    write(&root.join("submission/slurm-job-id.txt"), &format!("{job_id}\n"))?;
    write(&root.join("result/evidence.json"), "{\"status\":\"failed\"}\n")?;
    write(
        &root.join("result/slurm-accounting.json"),
        &format!("{{\"slurm_job_id\":\"{job_id}\"}}\n"),
    )?;
    let files = [
        "submission/config-pipeline.yml",
        "submission/slurm-job-id.txt",
        "result/evidence.json",
        "result/slurm-accounting.json",
    ]
    .iter()
    .map(|relative| file_record(&root, relative))
    .collect::<Result<Vec<_>>>()?;
    let superseded = scope == "superseded";
    let archive = json!({
        "schema_version": if superseded {
            "bijux.container.superseded_run_archive.v1"
        } else {
            "bijux.container.failed_run_archive.v1"
        },
        "run_id": run_id,
        "tool": tool_id,
        "slurm_job_id": job_id,
        "disposition": if superseded { Some("superseded") } else { None },
        "evidence_sha256": bijux_dna_infra::hash_file_sha256(&root.join("result/evidence.json"))?,
        "accounting_sha256": bijux_dna_infra::hash_file_sha256(
            &root.join("result/slurm-accounting.json")
        )?,
        "files": files,
    });
    bijux_dna_infra::atomic_write_json(&root.join("archive.json"), &archive)?;
    Ok(root)
}

fn qualification_attempt(study: &Path) -> Result<()> {
    let root = study.join("containers/evidence/failures/run-1139-lighter-slurm-3458866");
    write(
        &root.join("submission/config-pipeline.yml"),
        "run:\n  id: run-1139\n  kind: qualification\n  tool: lighter\n",
    )
}

#[test]
fn migration_apply_verify_and_rollback_preserve_exact_inventory() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let study = temp.path().join("STUDY");
    fs::create_dir_all(&study)?;
    let failed = historical_attempt(&study, "failures", "run-2043", "seqfu", "3459974")?;
    let superseded = historical_attempt(&study, "superseded", "run-2001", "bcftools", "3459727")?;
    let plural_failure = historical_attempt(&study, "failures", "run-2042", "seqkit", "3460229")?;
    let plural_failure_target =
        study.join("results/benchmark-attempts/failures").join("run-2042-seqkit-slurm-3460229");
    let plural_failure_parent = plural_failure_target
        .parent()
        .ok_or_else(|| anyhow::anyhow!("plural failure target has no parent"))?;
    fs::create_dir_all(plural_failure_parent)?;
    fs::rename(&plural_failure, &plural_failure_target)?;
    qualification_attempt(&study)?;

    let plan = plan_attempt_migration(&study, "2026-08-04T00:00:00Z".to_string())?;
    assert_eq!(plan.entries.len(), 3);
    assert_eq!(plan.exclusions.len(), 1);
    assert!(plan.entries.iter().any(|entry| {
        entry.run_id == "run-2001" && entry.disposition == BenchmarkAttemptDisposition::Superseded
    }));
    assert!(plan.entries.iter().any(|entry| {
        entry.run_id == "run-2043" && entry.disposition == BenchmarkAttemptDisposition::Failed
    }));
    let plan_path = study.join("evidence/migration-plan.json");
    let ledger_path = study.join("evidence/migration-ledger.json");
    write_attempt_migration_plan(&plan_path, &plan)?;

    let ledger = apply_attempt_migration(
        &study,
        &plan_path,
        &ledger_path,
        "2026-08-04T00:01:00Z".to_string(),
    )?;
    assert!(ledger
        .entries
        .iter()
        .all(|entry| entry.state == BenchmarkAttemptMigrationState::Applied));
    assert!(!failed.exists());
    assert!(!superseded.exists());
    assert!(!plural_failure_target.exists());
    let verification = verify_attempt_migration(&study, &plan_path, &ledger_path)?;
    assert_eq!(verification.entries.len(), 3);
    assert!(verification.entries.iter().all(|entry| entry.current_location == entry.target));

    let rolled_back = rollback_attempt_migration(
        &study,
        &plan_path,
        &ledger_path,
        "2026-08-04T00:02:00Z".to_string(),
    )?;
    assert!(rolled_back
        .entries
        .iter()
        .all(|entry| entry.state == BenchmarkAttemptMigrationState::RolledBack));
    assert!(failed.is_dir());
    assert!(superseded.is_dir());
    assert!(plural_failure_target.is_dir());
    let verification = verify_attempt_migration(&study, &plan_path, &ledger_path)?;
    assert!(verification.entries.iter().all(|entry| entry.current_location == entry.source));
    Ok(())
}

#[test]
fn migration_verify_and_rollback_refuse_mutated_target() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let study = temp.path().join("STUDY");
    fs::create_dir_all(&study)?;
    historical_attempt(&study, "failures", "run-2043", "seqfu", "3459974")?;
    let plan = plan_attempt_migration(&study, "2026-08-04T00:00:00Z".to_string())?;
    let plan_path = study.join("evidence/migration-plan.json");
    let ledger_path = study.join("evidence/migration-ledger.json");
    write_attempt_migration_plan(&plan_path, &plan)?;
    apply_attempt_migration(&study, &plan_path, &ledger_path, "2026-08-04T00:01:00Z".to_string())?;
    let target = study.join(
        "results/benchmark-attempts/failed/run-2043-seqfu-slurm-3459974/result/evidence.json",
    );
    fs::write(&target, "mutated\n")?;

    let verify_error = verify_attempt_migration(&study, &plan_path, &ledger_path)
        .err()
        .map(|error| error.to_string());
    assert!(verify_error.is_some_and(|error| error.contains("inventory changed")));
    let rollback_error = rollback_attempt_migration(
        &study,
        &plan_path,
        &ledger_path,
        "2026-08-04T00:02:00Z".to_string(),
    )
    .err()
    .map(|error| error.to_string());
    assert!(rollback_error.is_some_and(|error| error.contains("inventory changed")));
    Ok(())
}
