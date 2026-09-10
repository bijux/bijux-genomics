use crate::execution::output::prepare_output_root;
use anyhow::Result;
use serde_json::json;
use std::path::{Path, PathBuf};

fn report(repository: &Path, root: &Path, outputs: &serde_json::Value) -> Result<PathBuf> {
    let path = root.join("summary.json");
    bijux_dna_infra::atomic_write_json(
        &path,
        &json!({
            "schema_version": "workflow.v1", "pipeline_id": "test-workflow",
            "output_path": path.strip_prefix(repository)?.display().to_string(),
            "rows": [{"outputs": outputs, "evidence_path": null}]
        }),
    )?;
    Ok(path)
}

#[test]
fn unrelated_files_survive_first_admission_and_declared_output_retirement() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let repository = temp.path().canonicalize()?;
    let root = repository.join("output");
    bijux_dna_infra::ensure_dir(&root)?;
    let notes = root.join("notes.txt");
    std::fs::write(&notes, "user notes")?;
    let summary = root.join("summary.json");
    prepare_output_root(&repository, &root, &summary, "test-workflow", "workflow.v1")?;
    let generated = root.join("artifacts/stage/result.json");
    let extra = root.join("artifacts/stage/research.txt");
    bijux_dna_infra::atomic_write_bytes(&generated, b"generated")?;
    std::fs::write(&extra, "research")?;
    report(&repository, &root, &json!({"result": "output/artifacts/stage/result.json"}))?;
    prepare_output_root(&repository, &root, &summary, "test-workflow", "workflow.v1")?;
    assert!(!generated.exists());
    assert!(!summary.exists());
    assert_eq!(std::fs::read_to_string(notes)?, "user notes");
    assert_eq!(std::fs::read_to_string(extra)?, "research");
    Ok(())
}

#[test]
fn every_declaration_is_admitted_before_any_retirement() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let repository = temp.path().canonicalize()?;
    let root = repository.join("output");
    let generated = root.join("artifacts/result.json");
    bijux_dna_infra::atomic_write_bytes(&generated, b"previous")?;
    let summary = report(
        &repository,
        &root,
        &json!({"a": "output/artifacts/result.json", "z": "output/../source.txt"}),
    )?;
    assert!(
        prepare_output_root(&repository, &root, &summary, "test-workflow", "workflow.v1").is_err()
    );
    assert_eq!(std::fs::read(&generated)?, b"previous");
    assert!(summary.is_file());
    assert!(prepare_output_root(&repository, &root, &summary, "different", "workflow.v1").is_err());
    assert_eq!(std::fs::read(&generated)?, b"previous");
    Ok(())
}

#[test]
fn unclaimed_artifacts_and_repository_root_are_refused() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let repository = temp.path().canonicalize()?;
    let root = repository.join("output");
    let unclaimed = root.join("artifacts/unclaimed.txt");
    bijux_dna_infra::atomic_write_bytes(&unclaimed, b"keep")?;
    assert!(prepare_output_root(
        &repository,
        &root,
        &root.join("summary.json"),
        "test-workflow",
        "workflow.v1"
    )
    .is_err());
    assert!(prepare_output_root(
        &repository,
        &repository,
        &repository.join("summary.json"),
        "test-workflow",
        "workflow.v1"
    )
    .is_err());
    assert_eq!(std::fs::read(unclaimed)?, b"keep");
    Ok(())
}

#[cfg(unix)]
#[test]
fn symbolic_output_declarations_do_not_retire_external_files() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let repository = temp.path().canonicalize()?;
    let root = repository.join("output");
    bijux_dna_infra::ensure_dir(root.join("artifacts"))?;
    let external = repository.join("source.txt");
    std::fs::write(&external, "source")?;
    std::os::unix::fs::symlink(&external, root.join("artifacts/alias.txt"))?;
    let summary = report(&repository, &root, &json!({"alias": "output/artifacts/alias.txt"}))?;
    assert!(
        prepare_output_root(&repository, &root, &summary, "test-workflow", "workflow.v1").is_err()
    );
    assert_eq!(std::fs::read_to_string(external)?, "source");
    assert!(summary.is_file());
    Ok(())
}
