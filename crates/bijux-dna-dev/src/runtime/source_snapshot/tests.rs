use super::*;

fn source_fixture() -> Result<(tempfile::TempDir, Workspace)> {
    let root = tempfile::tempdir()?;
    let source = Workspace { root: root.path().join("source") };
    std::fs::create_dir(&source.root)?;
    git(&source, &["init", "--quiet", "--initial-branch=main"])?;
    git(&source, &["config", "commit.gpgsign", "false"])?;
    git(&source, &["config", "core.filemode", "true"])?;
    git(&source, &["config", "user.email", "fixture@example.invalid"])?;
    git(&source, &["config", "user.name", "Source snapshot fixture"])?;
    std::fs::write(source.path("unchanged.txt"), b"original content\n")?;
    std::fs::write(source.path("modified.txt"), b"before\n")?;
    std::fs::write(source.path("deleted.txt"), b"remove me\n")?;
    std::fs::write(source.path("binary.bin"), [0, 1, 2, 255])?;
    git(&source, &["add", "--", "."])?;
    git(&source, &["commit", "--quiet", "-m", "fixture: establish tracked source"])?;
    Ok((root, source))
}

#[test]
fn clean_snapshot_is_pinned_and_isolated_without_untracked_evidence() -> Result<()> {
    let (root, source) = source_fixture()?;
    std::fs::write(source.path("untracked.txt"), b"local evidence\n")?;
    let snapshot = TrackedSourceSnapshot::capture(&source)?;
    assert!(snapshot.patch.is_empty());
    assert_eq!(
        snapshot.patch_sha256(),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
    std::fs::write(source.path("unchanged.txt"), b"later commit\n")?;
    git(&source, &["commit", "--quiet", "-am", "fixture: advance live source"])?;
    let destination = root.path().join("consumer");
    snapshot.materialize(&destination)?;
    assert_eq!(std::fs::read(destination.join("unchanged.txt"))?, b"original content\n");
    assert!(!destination.join("untracked.txt").exists());
    assert!(destination.join("artifacts").is_dir());
    let isolated = Workspace { root: destination.clone() };
    assert_eq!(
        String::from_utf8(git(&isolated, &["rev-parse", "HEAD"])?)?.trim(),
        snapshot.commit()
    );
    std::fs::write(destination.join("unchanged.txt"), b"consumer mutation\n")?;
    assert_eq!(std::fs::read(source.path("unchanged.txt"))?, b"later commit\n");
    Ok(())
}

#[test]
fn dirty_snapshot_preserves_binary_additions_deletions_and_captured_content() -> Result<()> {
    let (root, source) = source_fixture()?;
    std::fs::write(source.path("modified.txt"), b"captured change\n")?;
    std::fs::write(source.path("binary.bin"), [255, 0, 13, 37, 0, 255])?;
    std::fs::remove_file(source.path("deleted.txt"))?;
    std::fs::write(source.path("added file.txt"), b"staged addition\n")?;
    git(&source, &["add", "--", "added file.txt"])?;
    let index_before = git(&source, &["ls-files", "--stage", "-z"])?;
    let snapshot = TrackedSourceSnapshot::capture(&source)?;
    assert!(!snapshot.patch.is_empty());
    std::fs::write(source.path("modified.txt"), b"later edit\n")?;
    for name in ["archive", "consumer"] {
        let destination = root.path().join(name);
        snapshot.materialize(&destination)?;
        assert_eq!(std::fs::read(destination.join("modified.txt"))?, b"captured change\n");
        assert_eq!(std::fs::read(destination.join("binary.bin"))?, [255, 0, 13, 37, 0, 255]);
        assert_eq!(std::fs::read(destination.join("added file.txt"))?, b"staged addition\n");
        assert!(!destination.join("deleted.txt").exists());
        assert!(!destination.join(".git/bijux-tracked-source.patch").exists());
    }
    assert_eq!(git(&source, &["ls-files", "--stage", "-z"])?, index_before);
    assert_eq!(std::fs::read(source.path("modified.txt"))?, b"later edit\n");
    Ok(())
}

#[test]
fn staged_deletion_does_not_copy_the_remaining_untracked_file() -> Result<()> {
    let (root, source) = source_fixture()?;
    git(&source, &["rm", "--cached", "--", "deleted.txt"])?;
    let snapshot = TrackedSourceSnapshot::capture(&source)?;
    let destination = root.path().join("consumer");
    snapshot.materialize(&destination)?;
    assert!(!destination.join("deleted.txt").exists());
    assert_eq!(std::fs::read(source.path("deleted.txt"))?, b"remove me\n");
    Ok(())
}

#[test]
fn existing_destination_is_rejected_without_removing_evidence() -> Result<()> {
    let (root, source) = source_fixture()?;
    let snapshot = TrackedSourceSnapshot::capture(&source)?;
    let destination = root.path().join("consumer");
    std::fs::create_dir(&destination)?;
    std::fs::write(destination.join("evidence.txt"), b"retain\n")?;
    assert!(snapshot.materialize(&destination).is_err());
    assert_eq!(std::fs::read(destination.join("evidence.txt"))?, b"retain\n");
    Ok(())
}

#[cfg(unix)]
#[test]
fn snapshot_preserves_symlink_targets_and_executable_modes() -> Result<()> {
    use std::os::unix::fs::{symlink, PermissionsExt};

    let (root, source) = source_fixture()?;
    symlink("unchanged.txt", source.path("link"))?;
    git(&source, &["add", "--", "link"])?;
    git(&source, &["commit", "--quiet", "-m", "fixture: track source link"])?;
    std::fs::remove_file(source.path("link"))?;
    symlink("modified.txt", source.path("link"))?;
    std::fs::set_permissions(source.path("modified.txt"), std::fs::Permissions::from_mode(0o755))?;
    let snapshot = TrackedSourceSnapshot::capture(&source)?;
    let destination = root.path().join("consumer");
    snapshot.materialize(&destination)?;
    assert_eq!(std::fs::read_link(destination.join("link"))?, Path::new("modified.txt"));
    assert_ne!(
        std::fs::metadata(destination.join("modified.txt"))?.permissions().mode() & 0o111,
        0
    );
    assert_eq!(std::fs::read(source.path("modified.txt"))?, b"before\n");
    Ok(())
}
