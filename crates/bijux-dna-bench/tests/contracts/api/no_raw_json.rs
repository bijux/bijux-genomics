use std::fs;
use std::io;
use std::path::{Path, PathBuf};

fn collect_rs_files(root: &Path, files: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        let kind = entry.file_type()?;
        if kind.is_symlink() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("source entry must not be a symbolic link: {}", path.display()),
            ));
        }
        if kind.is_dir() {
            collect_rs_files(&path, files)?;
        } else if kind.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            files.push(path);
        }
    }
    Ok(())
}

fn raw_json_offenders(src_dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    collect_rs_files(src_dir, &mut files)?;
    let mut offenders = Vec::new();
    for file in files {
        let relative = file.strip_prefix(src_dir).map_err(io::Error::other)?;
        let contents = fs::read_to_string(&file)?;
        let production = bijux_dna_testkit::workspace_support::strip_rust_test_modules(&contents);
        if production.contains("serde_json::Value")
            && !relative.starts_with("repo")
            && !relative.starts_with("artifacts")
        {
            offenders.push(relative.to_path_buf());
        }
    }
    offenders.sort();
    Ok(offenders)
}

#[test]
fn serde_json_value_is_confined_to_repo_and_artifacts() {
    let manifest_dir = crate::support::crate_root("bijux-dna-bench").expect("bench crate root");
    let offenders = raw_json_offenders(&manifest_dir.join("src"))
        .expect("scan benchmark production source without unreadable inputs");
    assert!(offenders.is_empty(), "serde_json::Value outside repo/artifacts: {offenders:?}");
}

fn source_fixture() -> anyhow::Result<(tempfile::TempDir, PathBuf)> {
    let parent = crate::support::repo_root()?.join("artifacts/tests/bench-raw-json-ownership");
    fs::create_dir_all(&parent)?;
    let owner = tempfile::Builder::new().prefix("raw-json-ownership-").tempdir_in(parent)?;
    let src = owner.path().join("artifacts/crates/benchmark/src");
    fs::create_dir_all(&src)?;
    Ok((owner, src))
}

fn write_source(src: &Path, relative: &str, contents: &str) -> io::Result<PathBuf> {
    let path = src.join(relative);
    fs::create_dir_all(path.parent().expect("source fixture parent"))?;
    fs::write(&path, contents)?;
    Ok(path)
}

#[test]
fn raw_json_ownership_is_relative_to_src_not_checkout_ancestors() -> anyhow::Result<()> {
    let (_owner, src) = source_fixture()?;
    let production = "pub fn value() -> serde_json::Value { serde_json::Value::Null }";
    for allowed in ["repo/loader.rs", "artifacts/writer.rs"] {
        write_source(&src, allowed, production)?;
    }
    assert!(raw_json_offenders(&src)?.is_empty());
    for denied in [
        "workflow/value.rs",
        "repository/value.rs",
        "artifacts_like/value.rs",
        "workflow/repo/value.rs",
        "workflow/artifacts/value.rs",
    ] {
        let path = write_source(&src, denied, production)?;
        // This checkout ancestor made the old absolute-path predicate vacuous.
        assert!(path.to_string_lossy().contains("/artifacts/"));
        assert_eq!(raw_json_offenders(&src)?, [PathBuf::from(denied)]);
        fs::remove_file(path)?;
    }
    Ok(())
}

#[test]
fn test_json_fixtures_do_not_hide_production_before_or_after_them() -> anyhow::Result<()> {
    let (_owner, src) = source_fixture()?;
    let fixture = r##"pub fn typed() {}
#[cfg(test)]
mod tests {
    fn wire() {
        let closing = "}";
        let raw = r#"} serde_json::Value"#;
        /* } /* { */ */
        assert_eq!(serde_json::json!([]), serde_json::Value::Array(Vec::new()));
    }
}
"##;
    let path = write_source(&src, "workflow/typed.rs", fixture)?;
    assert!(raw_json_offenders(&src)?.is_empty());
    let production = "pub fn raw() -> serde_json::Value { serde_json::Value::Null }";
    for source in [format!("{production}\n{fixture}"), format!("{fixture}\n{production}")] {
        fs::write(&path, source)?;
        assert_eq!(raw_json_offenders(&src)?, [PathBuf::from("workflow/typed.rs")]);
    }
    fs::write(path, format!("#![cfg(test)]\n{production}"))?;
    assert!(raw_json_offenders(&src)?.is_empty());
    Ok(())
}

#[test]
fn unreadable_or_unenumerable_source_cannot_pass() -> anyhow::Result<()> {
    let (_owner, src) = source_fixture()?;
    assert_eq!(
        raw_json_offenders(&src.join("missing"))
            .expect_err("unavailable source must refuse")
            .kind(),
        io::ErrorKind::NotFound
    );
    let path = write_source(&src, "workflow/broken.rs", "")?;
    assert!(raw_json_offenders(&path).is_err());
    fs::write(&path, [0xff, 0xfe])?;
    assert_eq!(
        raw_json_offenders(&src).expect_err("unavailable source must refuse").kind(),
        io::ErrorKind::InvalidData
    );
    #[cfg(unix)]
    {
        fs::remove_file(&path)?;
        std::os::unix::fs::symlink("missing", &path)?;
        assert_eq!(
            raw_json_offenders(&src).expect_err("unavailable source must refuse").kind(),
            io::ErrorKind::InvalidData
        );
    }
    Ok(())
}
