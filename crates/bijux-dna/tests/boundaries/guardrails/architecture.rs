use std::fs;
use std::path::{Path, PathBuf};

fn collect_rs_files(root: &Path, files: &mut Vec<PathBuf>) -> std::io::Result<()> {
    let root_type = fs::symlink_metadata(root)?.file_type();
    if !root_type.is_dir() || root_type.is_symlink() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("CLI source root must be a directory without redirection: {}", root.display()),
        ));
    }
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        let path = entry.path();
        if kind.is_symlink() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("CLI source entry must not redirect: {}", path.display()),
            ));
        }
        if kind.is_dir() {
            collect_rs_files(&path, files)?;
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
            if !kind.is_file() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("CLI Rust source must be a regular file: {}", path.display()),
                ));
            }
            files.push(path);
        }
    }
    Ok(())
}

fn assert_no_imports(dir: &str, forbidden: &[&str]) {
    let mut files = Vec::new();
    collect_rs_files(Path::new(dir), &mut files)
        .unwrap_or_else(|error| panic!("scan required CLI source {dir}: {error}"));
    assert!(!files.is_empty(), "required CLI source contains no Rust files: {dir}");
    files.sort();
    for file in files {
        let contents = fs::read_to_string(&file)
            .unwrap_or_else(|error| panic!("read required CLI source {}: {error}", file.display()));
        for needle in forbidden {
            assert!(
                !contents.contains(needle),
                "forbidden import in {}: {}",
                file.display(),
                needle
            );
        }
    }
}

fn cli_source_root() -> anyhow::Result<PathBuf> {
    super::support::crate_src("bijux-dna")
}

#[test]
fn cli_does_not_import_engine_internals() {
    let source =
        cli_source_root().unwrap_or_else(|error| panic!("resolve CLI source authority: {error}"));
    let source = source.to_str().unwrap_or_else(|| panic!("CLI source authority is not UTF-8"));
    assert_no_imports(
        source,
        &[
            "bijux_dna_engine::internal",
            "bijux_dna_engine::core::",
            "bijux_dna_engine::foundation::",
        ],
    );
}

fn assert_source_refused(source: &Path) {
    let rendered = source.to_string_lossy();
    let outcome = std::panic::catch_unwind(|| {
        assert_no_imports(&rendered, &["bijux_dna_engine::internal"]);
    });
    assert!(
        outcome.is_err(),
        "required CLI source must not be silently skipped: {}",
        source.display()
    );
}

#[test]
fn cli_import_guard_uses_runtime_declared_owner_source() -> anyhow::Result<()> {
    let expected = super::support::crate_src("bijux-dna")?;
    let actual = cli_source_root()?;
    assert_eq!(fs::canonicalize(&actual)?, fs::canonicalize(&expected)?);
    Ok(())
}

#[test]
fn cli_import_guard_accepts_clean_recursive_source() -> std::io::Result<()> {
    let owner = bijux_dna_testkit::tempdir_for("CLI clean import source");
    fs::create_dir(owner.path().join("commands"))?;
    fs::write(owner.path().join("lib.rs"), "pub mod commands;\n")?;
    fs::write(owner.path().join("commands/entrypoint.rs"), "pub fn entry() {}\n")?;
    assert_no_imports(
        &owner.path().to_string_lossy(),
        &[
            "bijux_dna_engine::internal",
            "bijux_dna_engine::core::",
            "bijux_dna_engine::foundation::",
        ],
    );
    Ok(())
}

#[test]
fn cli_import_guard_refuses_each_forbidden_engine_boundary() -> std::io::Result<()> {
    let owner = bijux_dna_testkit::tempdir_for("CLI forbidden import source");
    fs::create_dir(owner.path().join("commands"))?;
    for forbidden in
        ["bijux_dna_engine::internal", "bijux_dna_engine::core::", "bijux_dna_engine::foundation::"]
    {
        fs::write(
            owner.path().join("commands/entrypoint.rs"),
            format!(
                "use {forbidden}{}PrivateContract;\n",
                if forbidden.ends_with("::") { "" } else { "::" }
            ),
        )?;
        let rendered = owner.path().to_string_lossy();
        let outcome = std::panic::catch_unwind(|| assert_no_imports(&rendered, &[forbidden]));
        assert!(outcome.is_err(), "the actual guard must refuse {forbidden}");
    }
    Ok(())
}

#[test]
fn cli_import_guard_refuses_missing_or_non_directory_source() -> std::io::Result<()> {
    let owner = bijux_dna_testkit::tempdir_for("CLI source root admission");
    assert_source_refused(&owner.path().join("missing"));
    fs::write(owner.path().join("regular-file"), b"not a directory")?;
    assert_source_refused(&owner.path().join("regular-file"));
    Ok(())
}

#[test]
fn cli_import_guard_refuses_empty_source() {
    let owner = bijux_dna_testkit::tempdir_for("CLI empty source admission");
    assert_source_refused(owner.path());
}

#[test]
fn cli_import_guard_refuses_unreadable_recursive_rust_source() -> std::io::Result<()> {
    let owner = bijux_dna_testkit::tempdir_for("CLI source decoding admission");
    fs::write(owner.path().join("lib.rs"), "pub mod commands;\n")?;
    fs::create_dir(owner.path().join("commands"))?;
    fs::write(owner.path().join("commands/entrypoint.rs"), [0xff])?;
    assert_source_refused(owner.path());
    Ok(())
}

#[cfg(unix)]
#[test]
fn cli_import_guard_refuses_redirected_source() -> std::io::Result<()> {
    let owner = bijux_dna_testkit::tempdir_for("CLI source redirect admission");
    fs::write(owner.path().join("lib.rs"), "pub fn entry() {}\n")?;
    std::os::unix::fs::symlink(owner.path().join("missing"), owner.path().join("commands"))?;
    assert_source_refused(owner.path());
    Ok(())
}
