#![cfg(test)]

use super::{inspect_layering, CoreLayeringSources};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

fn declared_repository(root: &Path) -> PathBuf {
    root.join("declared/source/authority/workspace")
}

fn physical_fixture_layers(root: &Path) -> CoreLayeringSources {
    let source = declared_repository(root).join("crates/bijux-dna-core/src");
    CoreLayeringSources { foundation: source.join("foundation"), contract: source.join("contract") }
}

fn source_fixture(root: &Path) -> Result<CoreLayeringSources> {
    let repository = declared_repository(root);
    let src = repository.join("crates/bijux-dna-core/src");
    std::fs::create_dir_all(src.join("foundation/numeric"))?;
    std::fs::create_dir_all(src.join("contract"))?;
    std::fs::write(src.join("foundation/mod.rs"), "pub mod numeric;\n")?;
    std::fs::write(src.join("foundation/numeric/value.rs"), "pub struct Value;\n")?;
    std::fs::write(src.join("contract/mod.rs"), "pub struct Contract;\n")?;
    Ok(CoreLayeringSources::from_repository(&repository))
}

#[test]
fn policy__boundaries__source_admission__declared_repository_selects_the_actual_core_layers(
) -> Result<()> {
    let owner = bijux_dna_testkit::tempdir_for("core-layering-source-authority");
    let sources = source_fixture(owner.path())?;
    let expected = declared_repository(owner.path()).join("crates/bijux-dna-core/src");
    assert_eq!(sources.foundation, expected.join("foundation"));
    assert_eq!(sources.contract, expected.join("contract"));
    Ok(())
}

#[test]
fn policy__boundaries__source_admission__clean_recursive_layers_are_physically_observed(
) -> Result<()> {
    let owner = bijux_dna_testkit::tempdir_for("core-layering-clean-source");
    let sources = source_fixture(owner.path())?;
    let observation = inspect_layering(&sources)?;
    let mut actual = observation.files;
    actual.sort();
    let src = declared_repository(owner.path()).join("crates/bijux-dna-core/src");
    let mut expected = vec![
        src.join("foundation/mod.rs"),
        src.join("foundation/numeric/value.rs"),
        src.join("contract/mod.rs"),
    ];
    expected.sort();
    assert_eq!(actual, expected);
    assert!(observation.offenders.is_empty());
    Ok(())
}

#[test]
fn policy__boundaries__source_admission__each_forbidden_boundary_keeps_its_layer_and_source_identity(
) -> Result<()> {
    let owner = bijux_dna_testkit::tempdir_for("core-layering-forbidden-source");
    let sources = source_fixture(owner.path())?;
    for (layer, needle) in [
        ("foundation", "crate::contract"),
        ("foundation", "crate::metrics::registry"),
        ("contract", "crate::foundation::invariants"),
    ] {
        let file = declared_repository(owner.path())
            .join("crates/bijux-dna-core/src")
            .join(layer)
            .join("violation.rs");
        std::fs::write(&file, format!("use {needle}::Forbidden;\n"))?;
        let observation = inspect_layering(&sources)?;
        assert_eq!(
            observation.offenders,
            vec![format!("{} imports forbidden {}", file.display(), needle)]
        );
        std::fs::remove_file(file)?;
    }
    Ok(())
}

#[test]
fn policy__boundaries__source_admission__opposite_layer_tokens_remain_permitted() -> Result<()> {
    let owner = bijux_dna_testkit::tempdir_for("core-layering-permitted-boundaries");
    source_fixture(owner.path())?;
    let sources = physical_fixture_layers(owner.path());
    let src = declared_repository(owner.path()).join("crates/bijux-dna-core/src");
    std::fs::write(src.join("foundation/mod.rs"), "use crate::foundation::invariants;\n")?;
    std::fs::write(
        src.join("contract/mod.rs"),
        "use crate::contract::run; use crate::metrics::registry;\n",
    )?;
    let observation = inspect_layering(&sources)?;
    assert_eq!(observation.files.len(), 3);
    assert!(observation.offenders.is_empty());
    Ok(())
}

fn assert_refused(sources: &CoreLayeringSources, expected_path: &Path) -> Result<()> {
    let error =
        inspect_layering(sources).err().context("required core source was silently skipped")?;
    assert!(
        format!("{error:#}").contains(&expected_path.display().to_string()),
        "source identity missing from acquisition error: {error:#}"
    );
    Ok(())
}

#[test]
fn policy__boundaries__source_admission__missing_and_non_directory_layers_refuse() -> Result<()> {
    for layer in ["foundation", "contract"] {
        let owner = bijux_dna_testkit::tempdir_for("core-layering-required-layer");
        source_fixture(owner.path())?;
        let sources = physical_fixture_layers(owner.path());
        let selected =
            declared_repository(owner.path()).join("crates/bijux-dna-core/src").join(layer);
        std::fs::remove_dir_all(&selected)?;
        assert_refused(&sources, &selected)?;
        std::fs::write(&selected, b"not a source directory")?;
        assert_refused(&sources, &selected)?;
    }
    Ok(())
}

#[test]
fn policy__boundaries__source_admission__empty_required_layers_refuse() -> Result<()> {
    for layer in ["foundation", "contract"] {
        let owner = bijux_dna_testkit::tempdir_for("core-layering-empty-layer");
        source_fixture(owner.path())?;
        let sources = physical_fixture_layers(owner.path());
        let selected =
            declared_repository(owner.path()).join("crates/bijux-dna-core/src").join(layer);
        std::fs::remove_dir_all(&selected)?;
        std::fs::create_dir(&selected)?;
        assert_refused(&sources, &selected)?;
    }
    Ok(())
}

#[test]
fn policy__boundaries__source_admission__recursive_decode_failure_names_the_source() -> Result<()> {
    let owner = bijux_dna_testkit::tempdir_for("core-layering-source-decoding");
    source_fixture(owner.path())?;
    let sources = physical_fixture_layers(owner.path());
    let file = declared_repository(owner.path())
        .join("crates/bijux-dna-core/src/foundation/numeric/value.rs");
    std::fs::write(&file, [0xff])?;
    assert_refused(&sources, &file)
}

#[cfg(unix)]
#[test]
fn policy__boundaries__source_admission__redirected_layer_or_entry_refuses() -> Result<()> {
    let owner = bijux_dna_testkit::tempdir_for("core-layering-source-redirection");
    source_fixture(owner.path())?;
    let sources = physical_fixture_layers(owner.path());
    let foreign = bijux_dna_testkit::tempdir_for("core-layering-foreign-source");
    let selected = declared_repository(owner.path()).join("crates/bijux-dna-core/src/contract");
    std::fs::remove_dir_all(&selected)?;
    std::os::unix::fs::symlink(foreign.path(), &selected)?;
    assert_refused(&sources, &selected)?;
    std::fs::remove_file(&selected)?;
    std::fs::create_dir(&selected)?;
    std::fs::write(selected.join("mod.rs"), "pub struct Contract;\n")?;
    let redirected = selected.join("redirected.rs");
    std::os::unix::fs::symlink(foreign.path().join("missing.rs"), &redirected)?;
    assert_refused(&sources, &redirected)
}
