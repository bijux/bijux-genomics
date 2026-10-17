#![cfg(test)]
use super::{inspect_snapshots, snapshot_directory};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

fn declared_repository(root: &Path) -> PathBuf {
    root.join("declared/source/authority/workspace")
}
fn fixture(root: &Path) -> Result<PathBuf> {
    let repository = declared_repository(root);
    std::fs::create_dir_all(repository.join("crates/bijux-dna-policies/tests/snapshots"))?;
    Ok(repository)
}
fn physical_namespace(repository: &Path) -> PathBuf {
    repository.join("crates/bijux-dna-policies/tests/snapshots")
}
fn assert_refused(directory: &Path) -> Result<()> {
    let error = inspect_snapshots(directory)
        .err()
        .context("required snapshot namespace was silently skipped")?;
    assert!(format!("{error:#}").contains(&directory.display().to_string()));
    Ok(())
}

#[test]
fn policy__boundaries__source_admission__declared_repository_selects_policy_snapshot_namespace(
) -> Result<()> {
    let owner = bijux_dna_testkit::tempdir_for("snapshot-namespace-authority");
    let repository = fixture(owner.path())?;
    assert_eq!(snapshot_directory(&repository), physical_namespace(&repository));
    Ok(())
}

#[test]
fn policy__boundaries__source_admission__empty_owned_snapshot_namespace_is_permitted() -> Result<()>
{
    let owner = bijux_dna_testkit::tempdir_for("snapshot-empty-namespace");
    let repository = fixture(owner.path())?;
    let observation = inspect_snapshots(&physical_namespace(&repository))?;
    assert!(observation.files.is_empty());
    assert!(observation.offenders.is_empty());
    Ok(())
}

#[test]
fn policy__boundaries__source_admission__tree_snapshot_name_is_detected_in_declared_namespace(
) -> Result<()> {
    let owner = bijux_dna_testkit::tempdir_for("snapshot-forbidden-name");
    let repository = fixture(owner.path())?;
    let file = physical_namespace(&repository).join("owned_tree_contract.snap");
    std::fs::write(&file, "tree snapshot")?;
    let observation = inspect_snapshots(&snapshot_directory(&repository))?;
    assert_eq!(observation.files, vec![file.clone()]);
    assert_eq!(observation.offenders, vec![file.display().to_string()]);
    Ok(())
}

#[test]
fn policy__boundaries__source_admission__snapshot_extension_and_immediate_scope_stay_unchanged(
) -> Result<()> {
    let owner = bijux_dna_testkit::tempdir_for("snapshot-permitted-names");
    let repository = fixture(owner.path())?;
    let directory = physical_namespace(&repository);
    for name in ["tree_contract.json", "schema.snap", "tree_contract.snap.backup"] {
        std::fs::write(directory.join(name), "allowed snapshot")?;
    }
    std::fs::create_dir(directory.join("nested"))?;
    std::fs::write(
        directory.join("nested/tree_contract.snap"),
        "not an immediate namespace member",
    )?;
    let observation = inspect_snapshots(&directory)?;
    let mut actual = observation.files;
    actual.sort();
    let mut expected = ["tree_contract.json", "schema.snap", "tree_contract.snap.backup", "nested"]
        .map(|name| directory.join(name))
        .to_vec();
    expected.sort();
    assert_eq!(actual, expected);
    assert!(observation.offenders.is_empty());
    Ok(())
}

#[test]
fn policy__boundaries__source_admission__missing_and_non_directory_snapshot_namespace_refuse(
) -> Result<()> {
    let owner = bijux_dna_testkit::tempdir_for("snapshot-required-namespace");
    let repository = fixture(owner.path())?;
    let directory = physical_namespace(&repository);
    std::fs::remove_dir(&directory)?;
    assert_refused(&directory)?;
    std::fs::write(&directory, "not a directory")?;
    assert_refused(&directory)
}

#[cfg(unix)]
#[test]
fn policy__boundaries__source_admission__redirected_snapshot_namespace_refuses() -> Result<()> {
    let owner = bijux_dna_testkit::tempdir_for("snapshot-namespace-redirection");
    let repository = fixture(owner.path())?;
    let directory = physical_namespace(&repository);
    let foreign = bijux_dna_testkit::tempdir_for("snapshot-foreign-namespace");
    std::fs::remove_dir(&directory)?;
    std::os::unix::fs::symlink(foreign.path(), &directory)?;
    assert_refused(&directory)
}
