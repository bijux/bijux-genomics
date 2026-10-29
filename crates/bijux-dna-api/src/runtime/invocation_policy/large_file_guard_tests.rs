use anyhow::{bail, Context, Result};

use super::unexpected_large_file_violation;

#[test]
fn vanished_observed_file_does_not_fail_the_guard() -> Result<()> {
    let root = tempfile::tempdir()?;
    let path = root.path().join("atomic-output");
    std::fs::write(&path, b"oversized")?;
    let observed = walkdir::WalkDir::new(&path)
        .into_iter()
        .next()
        .transpose()?
        .context("observe the atomic output before removal")?;
    std::fs::remove_file(&path)?;

    assert_eq!(unexpected_large_file_violation(observed.path(), &[], 8)?, None);
    Ok(())
}

#[test]
fn unexpected_oversized_file_is_rejected() -> Result<()> {
    let root = tempfile::tempdir()?;
    let path = root.path().join("unexpected-output");
    std::fs::write(&path, b"oversized")?;

    let violation = unexpected_large_file_violation(&path, &[], 8)?
        .context("unexpected oversized output must be reported")?;
    assert_eq!(violation, format!("{} (9 bytes)", path.canonicalize()?.display()));
    Ok(())
}

#[test]
fn declared_oversized_file_is_allowed() -> Result<()> {
    let root = tempfile::tempdir()?;
    let path = root.path().join("declared-output");
    std::fs::write(&path, b"oversized")?;
    let allowed = [path.canonicalize()?];

    assert_eq!(unexpected_large_file_violation(&path, &allowed, 8)?, None);
    Ok(())
}

#[test]
fn unexpected_file_at_the_size_limit_is_allowed() -> Result<()> {
    let root = tempfile::tempdir()?;
    let path = root.path().join("small-output");
    std::fs::write(&path, b"boundary")?;

    assert_eq!(unexpected_large_file_violation(&path, &[], 8)?, None);
    Ok(())
}

#[test]
fn filesystem_errors_other_than_missing_files_are_reported() -> Result<()> {
    let root = tempfile::tempdir()?;
    let file = root.path().join("regular-file");
    std::fs::write(&file, b"contents")?;
    let path = file.join("child");

    let Err(error) = unexpected_large_file_violation(&path, &[], 8) else {
        bail!("an invalid parent must not be treated as a vanished file");
    };
    let io_error = error.downcast::<std::io::Error>()?;
    assert_eq!(io_error.kind(), std::io::ErrorKind::NotADirectory);
    Ok(())
}
