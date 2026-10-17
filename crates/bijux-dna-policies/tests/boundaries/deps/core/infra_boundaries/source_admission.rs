#![cfg(test)]
use super::{infra_source_directory, inspect_infra};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

fn declared_repository(root: &Path) -> PathBuf {
    root.join("declared/source/authority/workspace")
}
fn fixture(root: &Path) -> Result<PathBuf> {
    let repository = declared_repository(root);
    let source = repository.join("crates/bijux-dna-infra/src");
    std::fs::create_dir_all(source.join("filesystem"))?;
    std::fs::write(source.join("lib.rs"), "pub mod filesystem;\n")?;
    std::fs::write(source.join("filesystem/value.rs"), "pub struct Value;\n")?;
    Ok(repository)
}
fn assert_refused(source: &Path, expected: &Path) -> Result<()> {
    let error =
        inspect_infra(source).err().context("required infra source was silently skipped")?;
    assert!(format!("{error:#}").contains(&expected.display().to_string()));
    Ok(())
}

#[test]
fn policy__boundaries__source_admission__declared_repository_selects_infra_source() -> Result<()> {
    let owner = bijux_dna_testkit::tempdir_for("infra-source-authority");
    let repository = fixture(owner.path())?;
    assert_eq!(infra_source_directory(&repository), repository.join("crates/bijux-dna-infra/src"));
    Ok(())
}

#[test]
fn policy__boundaries__source_admission__recursive_infra_source_is_physically_observed(
) -> Result<()> {
    let owner = bijux_dna_testkit::tempdir_for("infra-source-observation");
    let source = fixture(owner.path())?.join("crates/bijux-dna-infra/src");
    let observation = inspect_infra(&source)?;
    let mut expected = vec![source.join("lib.rs"), source.join("filesystem/value.rs")];
    expected.sort();
    let mut actual = observation.files;
    actual.sort();
    assert_eq!(actual, expected);
    assert!(observation.offenders.is_empty());
    Ok(())
}

#[test]
fn policy__boundaries__source_admission__each_infra_literal_is_detected_in_declared_source(
) -> Result<()> {
    let owner = bijux_dna_testkit::tempdir_for("infra-literal-source");
    let repository = fixture(owner.path())?;
    let source = infra_source_directory(&repository);
    let file = repository.join("crates/bijux-dna-infra/src/filesystem/violation.rs");
    for constructor in ["StageId::new", "ToolId::new"] {
        std::fs::write(
            &file,
            format!("fn make() {{ let _ = {constructor}(\"catalog.literal\"); }}\n"),
        )?;
        assert_eq!(inspect_infra(&source)?.offenders, vec![file.display().to_string()]);
    }
    Ok(())
}

#[test]
fn policy__boundaries__source_admission__missing_non_directory_and_empty_infra_source_refuse(
) -> Result<()> {
    let owner = bijux_dna_testkit::tempdir_for("infra-required-source");
    let source = fixture(owner.path())?.join("crates/bijux-dna-infra/src");
    std::fs::remove_dir_all(&source)?;
    assert_refused(&source, &source)?;
    std::fs::write(&source, "not a directory")?;
    assert_refused(&source, &source)?;
    std::fs::remove_file(&source)?;
    std::fs::create_dir(&source)?;
    assert_refused(&source, &source)
}

#[test]
fn policy__boundaries__source_admission__infra_decode_error_names_the_actual_source() -> Result<()>
{
    let owner = bijux_dna_testkit::tempdir_for("infra-source-decoding");
    let source = fixture(owner.path())?.join("crates/bijux-dna-infra/src");
    let file = source.join("filesystem/value.rs");
    std::fs::write(&file, [0xff])?;
    assert_refused(&source, &file)
}

#[cfg(unix)]
#[test]
fn policy__boundaries__source_admission__redirected_infra_source_or_entry_refuses() -> Result<()> {
    let owner = bijux_dna_testkit::tempdir_for("infra-source-redirection");
    let source = fixture(owner.path())?.join("crates/bijux-dna-infra/src");
    let foreign = bijux_dna_testkit::tempdir_for("infra-foreign-source");
    std::fs::remove_dir_all(&source)?;
    std::os::unix::fs::symlink(foreign.path(), &source)?;
    assert_refused(&source, &source)?;
    std::fs::remove_file(&source)?;
    std::fs::create_dir(&source)?;
    std::fs::write(source.join("lib.rs"), "pub struct Utility;\n")?;
    let redirected = source.join("redirected.rs");
    std::os::unix::fs::symlink(foreign.path().join("missing.rs"), &redirected)?;
    assert_refused(&source, &redirected)
}
