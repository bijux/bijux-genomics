#[test]
fn atomic_write_failure_does_not_clobber() -> anyhow::Result<()> {
    let dir = bijux_dna_infra::temp_dir("bijux")?;
    let target = dir.path().join("payload.json");
    bijux_dna_infra::atomic_write_bytes(&target, b"stable")?;
    let result =
        bijux_dna_infra::atomic_write_with(&target, |_| Err(std::io::Error::other("boom")));
    assert!(result.is_err());
    let data = std::fs::read_to_string(&target)?;
    assert_eq!(data, "stable");
    Ok(())
}

#[test]
fn bounded_read_rejects_files_larger_than_limit() -> anyhow::Result<()> {
    let dir = bijux_dna_infra::temp_dir("bijux")?;
    let path = dir.path().join("payload.bin");
    bijux_dna_infra::atomic_write_bytes(&path, b"abcdef")?;

    let exact = bijux_dna_infra::read_to_end_bounded(&path, 6)?;
    assert_eq!(exact, b"abcdef");

    let err = bijux_dna_infra::read_to_end_bounded(&path, 5)
        .err()
        .ok_or_else(|| anyhow::anyhow!("expected bounded read failure"))?;
    assert_eq!(err.kind, bijux_dna_infra::IoErrorKind::Corruption);
    Ok(())
}

#[test]
fn temp_dir_is_created() -> anyhow::Result<()> {
    let dir = bijux_dna_infra::temp_dir("bijux-dna-test")?;
    assert!(dir.path().exists());
    Ok(())
}

#[cfg(unix)]
#[test]
fn remove_path_if_exists_removes_broken_symlink() -> anyhow::Result<()> {
    use std::os::unix::fs::symlink;

    let dir = bijux_dna_infra::temp_dir("bijux")?;
    let link = dir.path().join("dangling-link");
    symlink(dir.path().join("missing-target"), &link)?;

    bijux_dna_infra::remove_path_if_exists(&link)?;

    assert!(std::fs::symlink_metadata(&link).is_err(), "broken symlink should be removed");
    Ok(())
}
