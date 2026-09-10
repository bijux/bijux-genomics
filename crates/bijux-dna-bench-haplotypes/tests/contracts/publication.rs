use crate::publication::{execution_parent, PublicationPlan, PublishedArtifactPaths, ReportOwner};
use anyhow::Result;
use bijux_dna_infra::{atomic_write_json, ensure_dir, hash_file_sha256};
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};

fn owner() -> ReportOwner<'static> {
    ReportOwner {
        report_name: "metrics.json",
        schema: "metrics.v1",
        stage: "phase",
        tool: "shapeit",
    }
}

fn execution(destination: &Path, name: &str, payload: &[u8]) -> Result<PathBuf> {
    let root = execution_parent(destination)?.join(name);
    ensure_dir(&root)?;
    fs::write(root.join("phased.vcf"), payload)?;
    atomic_write_json(
        &root.join("manifest.json"),
        &json!({
            "output": root.join("phased.vcf"),
            "argv": ["shapeit", "--output", root.join("phased.vcf").to_str().expect("utf8")]
        }),
    )?;
    atomic_write_json(
        &root.join("checksums.json"),
        &json!({
            "manifest": root.join("manifest.json"),
            "sha256": hash_file_sha256(&root.join("manifest.json"))?
        }),
    )?;
    atomic_write_json(
        &root.join("metrics.json"),
        &json!({
            "schema_version": "metrics.v1", "stage_id": "phase", "tool_id": "shapeit"
        }),
    )?;
    Ok(root)
}

#[test]
fn publication_preserves_native_paths_checksums_and_prior_downstream_references() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let repository = temp.path().canonicalize()?;
    let destination = repository.join("published");
    let first = execution(&destination, "first", b"original phased variants")?;
    let original_manifest = fs::read(first.join("manifest.json"))?;
    PublicationPlan::admit(
        destination.parent().expect("repository"),
        &first,
        &destination,
        &owner(),
    )?
    .publish()?;
    let second = execution(&destination, "second", b"replacement phased variants")?;
    PublicationPlan::admit(
        destination.parent().expect("repository"),
        &second,
        &destination,
        &owner(),
    )?
    .publish()?;
    assert_eq!(fs::read(first.join("manifest.json"))?, original_manifest);
    assert_eq!(fs::read(first.join("phased.vcf"))?, b"original phased variants");
    assert_eq!(fs::read(destination.join("phased.vcf"))?, b"replacement phased variants");
    for root in [&first, &second] {
        let checks: serde_json::Value =
            serde_json::from_slice(&fs::read(root.join("checksums.json"))?)?;
        let manifest_path = Path::new(checks["manifest"].as_str().expect("manifest path"));
        assert_eq!(hash_file_sha256(manifest_path)?, checks["sha256"]);
        let manifest: serde_json::Value = serde_json::from_slice(&fs::read(manifest_path)?)?;
        assert!(Path::new(manifest["output"].as_str().expect("output path")).is_file());
        assert_eq!(manifest["argv"][2], manifest["output"]);
    }
    let paths = PublishedArtifactPaths {
        repository: &repository,
        execution: &second,
        destination: &destination,
    };
    assert_eq!(paths.path(&second.join("phased.vcf"))?, "published/phased.vcf");
    assert!(paths.path(&repository.join("foreign")).is_err());
    Ok(())
}

#[test]
fn collisions_are_rejected_before_replacing_previous_files() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let destination = temp.path().canonicalize()?.join("published");
    let first = execution(&destination, "first", b"old")?;
    PublicationPlan::admit(
        destination.parent().expect("repository"),
        &first,
        &destination,
        &owner(),
    )?
    .publish()?;
    fs::write(destination.join("z-unknown.txt"), b"user evidence")?;
    let second = execution(&destination, "second", b"new")?;
    fs::write(second.join("z-unknown.txt"), b"collision")?;
    assert!(PublicationPlan::admit(
        destination.parent().expect("repository"),
        &second,
        &destination,
        &owner()
    )
    .is_err());
    assert_eq!(fs::read(destination.join("phased.vcf"))?, b"old");
    assert_eq!(fs::read(destination.join("z-unknown.txt"))?, b"user evidence");
    Ok(())
}

#[test]
fn obsolete_owned_stable_files_retire_while_unknown_evidence_survives() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let destination = temp.path().canonicalize()?.join("published");
    let first = execution(&destination, "first", b"old")?;
    ensure_dir(first.join("diagnostics"))?;
    fs::write(first.join("diagnostics/old.txt"), b"prior diagnosis")?;
    PublicationPlan::admit(
        destination.parent().expect("repository"),
        &first,
        &destination,
        &owner(),
    )?
    .publish()?;
    fs::write(destination.join("diagnostics/user.txt"), b"user evidence")?;
    let second = execution(&destination, "second", b"new")?;
    PublicationPlan::admit(
        destination.parent().expect("repository"),
        &second,
        &destination,
        &owner(),
    )?
    .publish()?;
    assert!(!destination.join("diagnostics/old.txt").exists());
    assert_eq!(fs::read(destination.join("diagnostics/user.txt"))?, b"user evidence");
    assert!(first.join("diagnostics/old.txt").is_file());
    Ok(())
}

#[test]
fn malformed_and_foreign_inventory_cannot_authorize_replacement_or_retirement() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let destination = temp.path().canonicalize()?.join("published");
    let first = execution(&destination, "first", b"old")?;
    PublicationPlan::admit(
        destination.parent().expect("repository"),
        &first,
        &destination,
        &owner(),
    )?
    .publish()?;
    let inventory = destination.join("haplotype-artifact-ownership.json");
    let original: serde_json::Value = serde_json::from_slice(&fs::read(&inventory)?)?;
    let second = execution(&destination, "second", b"new")?;
    for (field, value) in [
        ("files", json!(["../foreign"])),
        ("tool_id", json!("foreign")),
        ("files", json!(["/absolute"])),
    ] {
        let mut invalid = original.clone();
        invalid[field] = value;
        atomic_write_json(&inventory, &invalid)?;
        assert!(PublicationPlan::admit(
            destination.parent().expect("repository"),
            &second,
            &destination,
            &owner()
        )
        .is_err());
        assert_eq!(fs::read(destination.join("phased.vcf"))?, b"old");
    }
    Ok(())
}

#[cfg(unix)]
#[test]
fn symlinked_stable_artifact_cannot_redirect_publication() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let root = temp.path().canonicalize()?;
    let destination = root.join("published");
    let first = execution(&destination, "first", b"old")?;
    PublicationPlan::admit(
        destination.parent().expect("repository"),
        &first,
        &destination,
        &owner(),
    )?
    .publish()?;
    let foreign = root.join("foreign.txt");
    fs::write(&foreign, b"user evidence")?;
    fs::remove_file(destination.join("phased.vcf"))?;
    std::os::unix::fs::symlink(&foreign, destination.join("phased.vcf"))?;
    let second = execution(&destination, "second", b"new")?;
    assert!(PublicationPlan::admit(
        destination.parent().expect("repository"),
        &second,
        &destination,
        &owner()
    )
    .is_err());
    assert_eq!(fs::read(&foreign)?, b"user evidence");
    Ok(())
}

fn stage_result(root: &Path, realized_path: &str) -> Result<()> {
    atomic_write_json(
        &root.join("stage-result.json"),
        &json!({
            "schema_version": crate::stage_result::BENCH_STAGE_RESULT_SCHEMA_VERSION,
            "stage_id": "vcf.phasing", "tool": { "id": "shapeit" },
            "command": { "rendered": "shapeit --output original" },
            "runtime": { "mode": "local_smoke", "status": "succeeded", "started_at": "start", "finished_at": "end", "elapsed_seconds": 1.0, "exit_code": 0 },
            "resource_metrics": { "source": "not_available", "memory_mb": null, "cpu_threads": null },
            "outputs": [{ "artifact_id": "phased_vcf", "declared_path": "phased.vcf", "realized_path": realized_path, "role": "vcf_output", "optional": false, "exists": true }]
        }),
    )?;
    Ok(())
}

#[test]
fn stable_stage_result_projects_addresses_without_mutating_execution_record() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let repository = temp.path().canonicalize()?;
    let destination = repository.join("published");
    let run = execution(&destination, "first", b"phased variants")?;
    stage_result(&run, "published/executions/first/phased.vcf")?;
    let original = fs::read(run.join("stage-result.json"))?;
    PublicationPlan::admit(&repository, &run, &destination, &owner())?.publish()?;
    assert_eq!(fs::read(run.join("stage-result.json"))?, original);
    let published: serde_json::Value =
        serde_json::from_slice(&fs::read(destination.join("stage-result.json"))?)?;
    assert_eq!(published["outputs"][0]["realized_path"], "published/phased.vcf");
    assert_eq!(published["command"]["rendered"], "shapeit --output original");
    Ok(())
}

#[test]
fn escaping_stage_result_is_rejected_before_replacing_stable_outputs() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let repository = temp.path().canonicalize()?;
    let destination = repository.join("published");
    let first = execution(&destination, "first", b"old")?;
    PublicationPlan::admit(&repository, &first, &destination, &owner())?.publish()?;
    let second = execution(&destination, "second", b"new")?;
    for invalid in ["foreign.vcf", "published/executions/second/../../foreign.vcf"] {
        stage_result(&second, invalid)?;
        assert!(PublicationPlan::admit(&repository, &second, &destination, &owner()).is_err());
        assert_eq!(fs::read(destination.join("phased.vcf"))?, b"old");
    }
    Ok(())
}
