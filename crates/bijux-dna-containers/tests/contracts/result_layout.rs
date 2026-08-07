use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use bijux_dna_containers::api::{
    apply_result_layout, apply_superseded_result_archive, derive_result_layout_plan,
    derive_superseded_result_archive_plan, rollback_result_layout,
    rollback_superseded_result_archive, verify_result_layout, verify_superseded_result_archive,
    ResultLayoutPlanRequest, SupersededResultArchivePlanRequest,
};

#[test]
fn flat_layout_plan_apply_verify_and_rollback_preserve_bytes() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let (runs, result_root) = nested_result_fixture(temp.path())?;
    let plan_path = result_root.join(".governance/result-layout.json");
    let plan = derive_result_layout_plan(&ResultLayoutPlanRequest {
        runs_root: runs.clone(),
        result_root: result_root.clone(),
        created_at_utc: "2026-08-07T03:00:00Z".to_string(),
    })?;
    bijux_dna_infra::atomic_write_json(&plan_path, &plan)?;

    assert_eq!(apply_result_layout(&plan_path, &runs, &result_root, "2026-08-07T03:01:00Z")?, 1);
    assert_eq!(verify_result_layout(&plan_path, &runs, &result_root)?, 1);
    assert_eq!(fs::read(result_root.join("run-1001-samtools/sample.bam"))?, b"governed-bam");
    assert!(!result_root.join("samtools").exists());

    assert_eq!(rollback_result_layout(&plan_path, &runs, &result_root, "2026-08-07T03:02:00Z")?, 1);
    assert_eq!(fs::read(result_root.join("samtools/run-1001/sample.bam"))?, b"governed-bam");
    Ok(())
}

#[test]
fn flat_layout_plan_rejects_unrecorded_files() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let (runs, result_root) = nested_result_fixture(temp.path())?;
    fs::write(result_root.join("samtools/run-1001/untracked.log"), "not inventoried")?;

    assert!(derive_result_layout_plan(&ResultLayoutPlanRequest {
        runs_root: runs,
        result_root,
        created_at_utc: "2026-08-07T03:00:00Z".to_string(),
    })
    .is_err_and(|error| error.to_string().contains("inventory differs")));
    Ok(())
}

#[test]
fn superseded_archive_preserves_attempts_and_active_finals() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let (request, plan_path) = superseded_fixture(temp.path())?;
    let plan = derive_superseded_result_archive_plan(&request)?;
    bijux_dna_infra::atomic_write_json(&plan_path, &plan)?;

    assert_eq!(apply_superseded_result_archive(&plan_path, &request, "2026-08-07T03:01:00Z")?, 2);
    assert_eq!(verify_superseded_result_archive(&plan_path, &request)?, 2);
    assert_eq!(fs::read(request.archive_root.join("run-1001-samtools/sample.bam"))?, b"superseded");
    assert_eq!(child_names(&request.result_root)?, ["run-1002-samtools".to_string()]);

    assert_eq!(
        rollback_superseded_result_archive(&plan_path, &request, "2026-08-07T03:02:00Z")?,
        2
    );
    assert!(request.result_root.join("run-1001-samtools").is_dir());
    assert!(request.result_root.join("python-cache").is_dir());
    Ok(())
}

#[test]
fn superseded_archive_rejects_changed_result_bytes() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let (request, plan_path) = superseded_fixture(temp.path())?;
    let plan = derive_superseded_result_archive_plan(&request)?;
    bijux_dna_infra::atomic_write_json(&plan_path, &plan)?;
    fs::write(request.result_root.join("run-1001-samtools/sample.bam"), "changed")?;

    assert!(apply_superseded_result_archive(&plan_path, &request, "2026-08-07T03:01:00Z")
        .is_err_and(|error| error.to_string().contains("size changed")));
    Ok(())
}

fn nested_result_fixture(root: &Path) -> Result<(PathBuf, PathBuf)> {
    let runs = root.join("runs");
    let result_root = root.join("results/container-validation");
    let result = result_root.join("samtools/run-1001");
    fs::create_dir_all(&result)?;
    fs::write(result.join("sample.bam"), b"governed-bam")?;
    write_lock(
        &runs,
        "run-1001",
        "samtools",
        "results/container-validation/samtools/run-1001",
        &result.join("sample.bam"),
    )?;
    Ok((runs, result_root))
}

fn superseded_fixture(root: &Path) -> Result<(SupersededResultArchivePlanRequest, PathBuf)> {
    let runs = root.join("runs");
    let result_root = root.join("results/container-validation");
    let final_result = result_root.join("run-1002-samtools");
    let superseded = result_root.join("run-1001-samtools");
    let cache = result_root.join("python-cache");
    fs::create_dir_all(&final_result)?;
    fs::create_dir_all(&superseded)?;
    fs::create_dir_all(&cache)?;
    fs::write(final_result.join("sample.bam"), b"published")?;
    fs::write(superseded.join("sample.bam"), b"superseded")?;
    fs::write(cache.join("module.pyc"), b"cache")?;
    write_lock(
        &runs,
        "run-1002",
        "samtools",
        "results/container-validation/run-1002-samtools",
        &final_result.join("sample.bam"),
    )?;
    let ledger = runs.join("attempt-ledger.json");
    bijux_dna_infra::atomic_write_json(
        &ledger,
        &serde_json::json!({
            "schema_version": "bijux.container.run_attempt_ledger.v1",
            "attempts": [{
                "run_id": "run-1001",
                "tool": "samtools",
                "evidence_lock_sha256": "a".repeat(64),
                "superseded_by": "run-1002"
            }]
        }),
    )?;
    let archive_root = root.join("evidence/superseded-results");
    let plan_path = root.join("evidence/result-layout.json");
    Ok((
        SupersededResultArchivePlanRequest {
            runs_root: runs,
            attempt_ledger_path: ledger,
            result_root,
            archive_root,
            created_at_utc: "2026-08-07T03:00:00Z".to_string(),
        },
        plan_path,
    ))
}

fn write_lock(
    runs: &Path,
    run_id: &str,
    tool: &str,
    source_result: &str,
    file: &Path,
) -> Result<()> {
    let lock_root = runs.join(format!("{run_id}-{tool}"));
    fs::create_dir_all(&lock_root)?;
    bijux_dna_infra::atomic_write_json(
        &lock_root.join("evidence.lock.json"),
        &serde_json::json!({
            "run_id": run_id,
            "tool": tool,
            "source_result": source_result,
            "source_files": [{
                "path": "sample.bam",
                "size_bytes": file.metadata()?.len(),
                "sha256": bijux_dna_infra::hash_file_sha256(file)?
            }]
        }),
    )?;
    Ok(())
}

fn child_names(root: &Path) -> Result<Vec<String>> {
    let mut names = fs::read_dir(root)?
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect::<Vec<_>>();
    names.sort();
    Ok(names)
}
