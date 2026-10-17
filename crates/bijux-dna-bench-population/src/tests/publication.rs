#![cfg(test)]
//! Filesystem safety across legacy admission, inventory reuse and failed preflight.
use crate::execution::publication::{publish, ReportOwner};
use anyhow::Result;
use std::fs;
use std::path::Path;

fn owner() -> ReportOwner<'static> {
    ReportOwner { report_name: "pca.json", schema: "test.pca", stage: "vcf.pca", tool: "plink2" }
}
fn stage(path: &Path) -> Result<()> {
    fs::create_dir_all(path.join("artifacts"))?;
    fs::write(path.join("artifacts/result.tsv"), "new result")?;
    fs::write(
        path.join("pca.json"),
        r#"{"schema_version":"test.pca","stage_id":"vcf.pca","tool_id":"plink2"}"#,
    )?;
    Ok(())
}
#[test]
fn repeated_publication_retires_only_owned_files() -> Result<()> {
    let root = tempfile::tempdir()?;
    let output = root.path().join("published");
    let first = root.path().join("first");
    stage(&first)?;
    fs::write(first.join("obsolete.txt"), "old generated")?;
    publish(&first, &output, &owner())?;
    fs::write(output.join("notes.txt"), "retain my notes")?;
    fs::write(output.join("artifacts/user.tsv"), "retain my data")?;
    let second = root.path().join("second");
    stage(&second)?;
    publish(&second, &output, &owner())?;
    assert!(!output.join("obsolete.txt").exists());
    assert_eq!(fs::read_to_string(output.join("notes.txt"))?, "retain my notes");
    assert_eq!(fs::read_to_string(output.join("artifacts/user.tsv"))?, "retain my data");
    Ok(())
}
#[test]
fn legacy_publication_preserves_unclaimed_artifacts() -> Result<()> {
    let root = tempfile::tempdir()?;
    let output = root.path().join("published");
    stage(&output)?;
    fs::write(output.join("legacy.log"), "retained without ownership proof")?;
    let staged = root.path().join("stage");
    stage(&staged)?;
    publish(&staged, &output, &owner())?;
    assert_eq!(fs::read_to_string(output.join("legacy.log"))?, "retained without ownership proof");
    assert!(output.join("population-artifact-ownership.json").is_file());
    Ok(())
}
#[test]
fn unclaimed_collision_is_rejected_before_any_replacement() -> Result<()> {
    let root = tempfile::tempdir()?;
    let output = root.path().join("published");
    fs::create_dir_all(output.join("artifacts"))?;
    fs::write(output.join("artifacts/result.tsv"), "user data")?;
    let staged = root.path().join("stage");
    stage(&staged)?;
    assert!(publish(&staged, &output, &owner()).is_err());
    assert_eq!(fs::read_to_string(output.join("artifacts/result.tsv"))?, "user data");
    assert!(!output.join("pca.json").exists());
    Ok(())
}
#[test]
fn escaping_prior_inventory_is_rejected_before_any_replacement() -> Result<()> {
    let root = tempfile::tempdir()?;
    let output = root.path().join("published");
    stage(&output)?;
    fs::write(root.path().join("external.txt"), "outside")?;
    fs::write(
        output.join("population-artifact-ownership.json"),
        r#"{"schema_version":"bijux.bench.population_artifact_ownership.v1","report_schema":"test.pca","stage_id":"vcf.pca","tool_id":"plink2","files":["artifacts/result.tsv","../external.txt"]}"#,
    )?;
    let staged = root.path().join("stage");
    stage(&staged)?;
    fs::write(staged.join("artifacts/result.tsv"), "replacement")?;
    assert!(publish(&staged, &output, &owner()).is_err());
    assert_eq!(fs::read_to_string(output.join("artifacts/result.tsv"))?, "new result");
    assert_eq!(fs::read_to_string(root.path().join("external.txt"))?, "outside");
    Ok(())
}
#[test]
fn foreign_legacy_owner_is_rejected() -> Result<()> {
    let root = tempfile::tempdir()?;
    let output = root.path().join("published");
    stage(&output)?;
    fs::write(
        output.join("pca.json"),
        r#"{"schema_version":"test.pca","stage_id":"vcf.roh","tool_id":"plink2"}"#,
    )?;
    let staged = root.path().join("stage");
    stage(&staged)?;
    assert!(publish(&staged, &output, &owner()).is_err());
    assert!(!output.join("population-artifact-ownership.json").exists());
    Ok(())
}
#[cfg(unix)]
#[test]
fn symbolic_destination_is_rejected_without_touching_external_files() -> Result<()> {
    let root = tempfile::tempdir()?;
    let output = root.path().join("published");
    stage(&output)?;
    let external = root.path().join("external");
    fs::write(&external, "external")?;
    fs::remove_file(output.join("artifacts/result.tsv"))?;
    std::os::unix::fs::symlink(&external, output.join("artifacts/result.tsv"))?;
    let staged = root.path().join("stage");
    stage(&staged)?;
    assert!(publish(&staged, &output, &owner()).is_err());
    assert_eq!(fs::read_to_string(external)?, "external");
    Ok(())
}

#[test]
fn malformed_current_report_preserves_stable_population_artifacts() -> Result<()> {
    let root = tempfile::tempdir()?;
    let output = root.path().join("published");
    let first = root.path().join("first");
    stage(&first)?;
    publish(&first, &output, &owner())?;
    let original = ["pca.json", "artifacts/result.tsv", "population-artifact-ownership.json"]
        .into_iter()
        .map(|relative| {
            let path = output.join(relative);
            Ok((relative, fs::read(&path)?, fs::metadata(&path)?.permissions()))
        })
        .collect::<Result<Vec<_>>>()?;
    let staged = root.path().join("second");
    stage(&staged)?;
    fs::write(staged.join("artifacts/result.tsv"), b"replacement result")?;
    fs::write(staged.join("pca.json"), b"{invalid current report")?;
    assert!(
        publish(&staged, &output, &owner()).is_err(),
        "malformed current population report must be refused before publication"
    );
    for (relative, bytes, permissions) in original {
        let path = output.join(relative);
        assert_eq!(fs::read(&path)?, bytes, "prior artifact {relative}");
        assert_eq!(fs::metadata(path)?.permissions(), permissions);
    }
    assert_eq!(fs::read(staged.join("pca.json"))?, b"{invalid current report");
    assert_eq!(fs::read(staged.join("artifacts/result.tsv"))?, b"replacement result");
    Ok(())
}
