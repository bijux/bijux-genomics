#![cfg(test)]
use super::{default_source_roots, inspect_default_sources};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

const OWNERS: [&str; 4] =
    ["bijux-dna-api", "bijux-dna", "bijux-dna-stages-fastq", "bijux-dna-stages-bam"];
fn declared_repository(root: &Path) -> PathBuf {
    root.join("declared/source/authority/workspace")
}
fn fixture(root: &Path) -> Result<PathBuf> {
    let repository = declared_repository(root);
    for owner in OWNERS {
        let source = repository.join("crates").join(owner).join("src");
        std::fs::create_dir_all(source.join("workflow"))?;
        std::fs::write(source.join("lib.rs"), "pub mod workflow;\n")?;
        std::fs::write(source.join("workflow/value.rs"), "pub struct Value;\n")?;
    }
    Ok(repository)
}
fn physical_sources(repository: &Path) -> [PathBuf; 4] {
    OWNERS.map(|owner| repository.join("crates").join(owner).join("src"))
}
fn assert_refused(sources: &[PathBuf], selected: &Path) -> Result<()> {
    let error = inspect_default_sources(sources)
        .err()
        .context("required default-policy source was silently skipped")?;
    assert!(format!("{error:#}").contains(&selected.display().to_string()));
    Ok(())
}

#[test]
fn policy__contracts__source_admission__declared_repository_selects_all_default_owners(
) -> Result<()> {
    let owner = bijux_dna_testkit::tempdir_for("default-policy-source-authority");
    let repository = fixture(owner.path())?;
    assert_eq!(default_source_roots(&repository), physical_sources(&repository));
    Ok(())
}

#[test]
fn policy__contracts__source_admission__every_recursive_default_owner_is_physically_observed(
) -> Result<()> {
    let owner = bijux_dna_testkit::tempdir_for("default-policy-source-observation");
    let repository = fixture(owner.path())?;
    let sources = physical_sources(&repository);
    let mut expected = sources
        .iter()
        .flat_map(|source| [source.join("lib.rs"), source.join("workflow/value.rs")])
        .collect::<Vec<_>>();
    expected.sort();
    let observation = inspect_default_sources(&sources)?;
    let mut actual = observation.files;
    actual.sort();
    assert_eq!(actual, expected);
    assert!(observation.offenders.is_empty());
    Ok(())
}

#[test]
fn policy__contracts__source_admission__both_default_predicates_detect_each_declared_owner(
) -> Result<()> {
    let owner = bijux_dna_testkit::tempdir_for("default-policy-forbidden-source");
    let repository = fixture(owner.path())?;
    let sources = default_source_roots(&repository);
    for source in physical_sources(&repository) {
        let file = source.join("workflow/violation.rs");
        for text in [
            "fn make() { let _ = NativeParams::default(); }",
            "fn make() -> NativeParams { Default::default() }",
        ] {
            std::fs::write(&file, text)?;
            assert_eq!(
                inspect_default_sources(&sources)?.offenders,
                vec![file.display().to_string()]
            );
        }
        std::fs::remove_file(file)?;
    }
    Ok(())
}

#[test]
fn policy__contracts__source_admission__each_missing_non_directory_and_empty_default_owner_refuses(
) -> Result<()> {
    for index in 0..OWNERS.len() {
        let owner = bijux_dna_testkit::tempdir_for("default-policy-required-source");
        let repository = fixture(owner.path())?;
        let sources = physical_sources(&repository);
        let selected = &sources[index];
        std::fs::remove_dir_all(selected)?;
        assert_refused(&sources, selected)?;
        std::fs::write(selected, "not a directory")?;
        assert_refused(&sources, selected)?;
        std::fs::remove_file(selected)?;
        std::fs::create_dir(selected)?;
        assert_refused(&sources, selected)?;
    }
    Ok(())
}

#[test]
fn policy__contracts__source_admission__default_predicate_scope_and_token_boundaries_stay_unchanged(
) -> Result<()> {
    let owner = bijux_dna_testkit::tempdir_for("default-policy-permitted-source");
    let repository = fixture(owner.path())?;
    let sources = physical_sources(&repository);
    let source = &sources[0];
    std::fs::write(
        source.join("workflow/value.rs"),
        "pub struct NativeParams; fn spelling() { let _ = NativeParams::defaultish(); }",
    )?;
    std::fs::write(
        source.join("workflow/other.rs"),
        "fn make() { let _: u8 = Default::default(); }",
    )?;
    assert!(inspect_default_sources(&sources)?.offenders.is_empty());
    Ok(())
}
