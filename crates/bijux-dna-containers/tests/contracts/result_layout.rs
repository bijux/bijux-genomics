#[path = "result_layout/resume_recovery.rs"]
mod resume_recovery;

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use bijux_dna_containers::api::{
    apply_result_layout, apply_superseded_result_archive, derive_result_layout_plan,
    derive_superseded_result_archive_plan, ensure_container_attempt_layout, rollback_result_layout,
    rollback_superseded_result_archive, verify_result_layout, verify_superseded_result_archive,
    ContainerAttemptIdentity, ResultLayoutPlanRequest, SupersededResultArchivePlanRequest,
};

fn container_attempt_identity() -> ContainerAttemptIdentity {
    ContainerAttemptIdentity {
        run_directory: "run-2050-fastq-trim-reads".to_string(),
        tool_id: "fastp".to_string(),
        campaign_id: "retained-tools-20260807".to_string(),
        attempt_id: "attempt-001".to_string(),
    }
}

#[test]
fn container_attempt_layout_uses_shared_owner_separated_contract() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let container_root = temp.path().join("container-attempts");
    let benchmark_root = temp.path().join("benchmark-attempts");
    let identity = container_attempt_identity();

    let first = ensure_container_attempt_layout(&container_root, &benchmark_root, &identity)?;
    assert_eq!(
        first.attempt_dir.strip_prefix(&container_root)?,
        Path::new("run-2050-fastq-trim-reads/fastp/attempts/retained-tools-20260807/attempt-001")
    );
    for directory in [
        &first.manifests_dir,
        &first.logs_dir,
        &first.outputs_dir,
        &first.accounting_dir,
        &first.validation_dir,
        &first.science_dir,
        &first.receipts_dir,
    ] {
        assert!(directory.is_dir());
    }
    let manifest: bijux_dna_infra::AttemptLayoutManifestV1 =
        serde_json::from_slice(&fs::read(&first.layout_manifest_path)?)?;
    assert_eq!(manifest.owner, bijux_dna_infra::AttemptEvidenceOwner::Containers);
    assert_eq!(
        bijux_dna_containers::api::container_attempt_layout_schema(),
        bijux_dna_infra::attempt_layout_schema()
    );

    fs::write(first.outputs_dir.join("image.sif.sha256"), b"immutable")?;
    let recovered = ensure_container_attempt_layout(&container_root, &benchmark_root, &identity)?;
    assert_eq!(recovered, first);
    assert_eq!(fs::read(recovered.outputs_dir.join("image.sif.sha256"))?, b"immutable");
    Ok(())
}

#[test]
fn container_attempt_layout_refuses_owner_overlap_before_mutation() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let shared_root = temp.path().join("shared-attempts");
    let identity = container_attempt_identity();

    let Err(error) = ensure_container_attempt_layout(&shared_root, &shared_root, &identity) else {
        anyhow::bail!("shared evidence ownership was accepted");
    };
    assert!(format!("{error:#}").contains("separate"));
    assert!(!shared_root.exists());
    Ok(())
}

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
    resume_recovery::require_recorded_rename_recovery(&runs, &result_root)?;
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

#[test]
fn flat_layout_rejects_identity_destination_and_count_drift_before_mutation() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let (runs, result_root) = nested_result_fixture(temp.path())?;
    let plan_path = temp.path().join("layout-plan.json");
    let plan = derive_result_layout_plan(&ResultLayoutPlanRequest {
        runs_root: runs.clone(),
        result_root: result_root.clone(),
        created_at_utc: "2026-08-07T03:00:00Z".to_string(),
    })?;
    bijux_dna_infra::atomic_write_json(&plan_path, &plan)?;
    let source = fs::read(&plan_path)?;
    let original: serde_json::Value = serde_json::from_slice(&source)?;
    for (pointer, value) in [
        ("/moves/0/target", serde_json::json!("run-1009-samtools")),
        ("/moves/0/source", serde_json::json!("samtools/run-1009")),
        ("/moves/0/run_id", serde_json::json!("run-1009")),
        ("/moves/0/tool", serde_json::json!("fastp")),
        ("/moves/0/file_count", serde_json::json!(2)),
        ("/run_count", serde_json::json!(2)),
        ("/file_count", serde_json::json!(2)),
        ("/size_bytes", serde_json::json!(0)),
    ] {
        let mut bad = original.clone();
        if pointer == "/moves/1/run_id" {
            bad["moves"][1]["run_id"] = value;
        } else {
            *bad.pointer_mut(pointer)
                .ok_or_else(|| anyhow::anyhow!("missing mutation field {pointer}"))? = value;
        }
        if pointer == "/moves/0/file_count" {
            bad["file_count"] = bad["moves"][0]["file_count"].clone();
        }
        let bytes = serde_json::to_vec(&bad)?;
        fs::write(&plan_path, &bytes)?;
        assert!(
            apply_result_layout(&plan_path, &runs, &result_root, "2026-08-07T03:01:00Z").is_err(),
            "{pointer}"
        );
        assert_eq!(fs::read(&plan_path)?, bytes, "failed admission rewrote the plan");
        assert_eq!(fs::read(result_root.join("samtools/run-1001/sample.bam"))?, b"governed-bam");
        assert_eq!(child_names(&result_root)?, ["samtools".to_string()]);
    }
    fs::write(&plan_path, source)?;
    Ok(())
}

#[test]
fn superseded_archive_rejects_forged_attempts_and_cache_destinations_before_mutation() -> Result<()>
{
    let temp = tempfile::tempdir()?;
    let (request, plan_path) = superseded_fixture(temp.path())?;
    let plan = derive_superseded_result_archive_plan(&request)?;
    let original = serde_json::to_value(&plan)?;
    bijux_dna_infra::atomic_write_json(&plan_path, &plan)?;
    for (pointer, value) in [
        ("/moves/0/target", serde_json::json!("run-1009-samtools")),
        ("/moves/0/run_id", serde_json::json!("run-1009")),
        ("/moves/0/superseded_by", serde_json::json!("run-1009")),
        ("/moves/0/evidence_lock_sha256", serde_json::json!("b".repeat(64))),
        ("/moves/1/target", serde_json::json!("unrelated-cache")),
        ("/moves/1/run_id", serde_json::json!("run-1009")),
        ("/move_count", serde_json::json!(3)),
        ("/size_bytes", serde_json::json!(0)),
    ] {
        let mut bad = original.clone();
        if pointer == "/moves/1/run_id" {
            bad["moves"][1]["run_id"] = value;
        } else {
            *bad.pointer_mut(pointer)
                .ok_or_else(|| anyhow::anyhow!("missing mutation field {pointer}"))? = value;
        }
        let bytes = serde_json::to_vec(&bad)?;
        fs::write(&plan_path, &bytes)?;
        assert!(
            apply_superseded_result_archive(&plan_path, &request, "2026-08-07T03:01:00Z").is_err(),
            "{pointer}"
        );
        assert_eq!(fs::read(&plan_path)?, bytes, "failed admission rewrote the plan");
        assert_eq!(
            fs::read(request.result_root.join("run-1001-samtools/sample.bam"))?,
            b"superseded"
        );
        assert_eq!(
            fs::read(request.result_root.join("run-1002-samtools/sample.bam"))?,
            b"published"
        );
        assert!(!request.archive_root.exists());
    }
    bijux_dna_infra::atomic_write_json(&plan_path, &plan)?;
    Ok(())
}

#[cfg(unix)]
#[test]
fn result_layout_rejects_linked_files_ancestors_locks_cycles_and_special_entries() -> Result<()> {
    use std::os::unix::fs::symlink;
    for location in ["file", "tool", "lock", "cycle", "fifo"] {
        let temp = tempfile::tempdir()?;
        let (runs, result_root) = nested_result_fixture(temp.path())?;
        let request = ResultLayoutPlanRequest {
            runs_root: runs.clone(),
            result_root: result_root.clone(),
            created_at_utc: "2026-08-07T03:00:00Z".into(),
        };
        let plan = derive_result_layout_plan(&request)?;
        let plan_path = result_root.join(".governance/result-layout.json");
        bijux_dna_infra::atomic_write_json(&plan_path, &plan)?;
        let original_plan = fs::read(&plan_path)?;
        let outside = temp.path().join("outside");
        fs::create_dir(&outside)?;
        fs::write(outside.join("guard"), b"unrelated")?;
        let source = result_root.join("samtools/run-1001");
        match location {
            "cycle" => symlink(&source, source.join("cycle"))?,
            "fifo" => {
                assert!(std::process::Command::new("mkfifo")
                    .arg(source.join("fifo"))
                    .status()?
                    .success());
            }
            _ => {
                let replaced = match location {
                    "file" => source.join("sample.bam"),
                    "tool" => result_root.join("samtools"),
                    _ => runs.join("run-1001-samtools/evidence.lock.json"),
                };
                fs::rename(&replaced, outside.join("entry"))?;
                symlink(outside.join("entry"), &replaced)?;
            }
        }
        let Err(error) =
            apply_result_layout(&plan_path, &runs, &result_root, "2026-08-07T03:01:00Z")
        else {
            anyhow::bail!("linked/special result evidence was accepted");
        };
        assert!(
            error.to_string().contains("link") || error.to_string().contains("regular"),
            "{location}: {error:#}"
        );
        assert!(derive_result_layout_plan(&request).is_err());
        assert!(!result_root.join("run-1001-samtools").exists());
        assert_eq!(String::from_utf8(fs::read(&plan_path)?)?, String::from_utf8(original_plan)?);
        assert_eq!(fs::read(outside.join("guard"))?, b"unrelated");
    }
    Ok(())
}

#[cfg(unix)]
#[test]
fn archive_apply_verify_and_rollback_reject_linked_destinations() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let (request, plan_path) = superseded_fixture(temp.path())?;
    let plan = derive_superseded_result_archive_plan(&request)?;
    bijux_dna_infra::atomic_write_json(&plan_path, &plan)?;
    let outside = temp.path().join("outside");
    fs::create_dir(&outside)?;
    fs::write(outside.join("guard"), b"unrelated")?;
    fs::create_dir_all(&request.archive_root)?;
    std::os::unix::fs::symlink(&outside, request.archive_root.join("run-1001-samtools"))?;
    for result in [
        apply_superseded_result_archive(&plan_path, &request, "2026-08-07T03:01:00Z"),
        verify_superseded_result_archive(&plan_path, &request),
        rollback_superseded_result_archive(&plan_path, &request, "2026-08-07T03:02:00Z"),
    ] {
        assert!(result.is_err_and(|error| error.to_string().contains("link")));
    }
    assert_eq!(fs::read(request.result_root.join("run-1001-samtools/sample.bam"))?, b"superseded");
    assert_eq!(fs::read(outside.join("guard"))?, b"unrelated");
    assert!(request.result_root.join("python-cache/module.pyc").exists());
    Ok(())
}
