//! Immutable tracked input shared by an archive producer and its isolated consumers.

use std::path::Path;

use anyhow::{ensure, Context, Result};

use super::{process::ProcessRunner, workspace::Workspace};

pub(crate) struct TrackedSourceSnapshot {
    source: Workspace,
    commit: String,
    patch: Vec<u8>,
}

impl TrackedSourceSnapshot {
    /// Capture tracked worktree content without changing the source index or copying evidence.
    pub(crate) fn capture(source: &Workspace) -> Result<Self> {
        let unresolved = git(source, &["ls-files", "--unmerged", "-z"])?;
        ensure!(unresolved.is_empty(), "source snapshot requires resolved index conflicts");
        let commit = String::from_utf8(git(source, &["rev-parse", "HEAD"])?)?.trim().to_owned();
        let patch = git(
            source,
            &[
                "diff",
                "--binary",
                "--full-index",
                "--no-ext-diff",
                "--no-textconv",
                "--no-renames",
                "--no-color",
                "--src-prefix=a/",
                "--dst-prefix=b/",
                &commit,
                "--",
            ],
        )?;
        Ok(Self { source: source.clone(), commit, patch })
    }

    pub(crate) fn commit(&self) -> &str {
        &self.commit
    }

    pub(crate) fn patch_sha256(&self) -> String {
        bijux_dna_infra::hash_bytes_sha256(&self.patch)
    }

    pub(crate) fn is_clean(&self) -> bool {
        self.patch.is_empty()
    }

    /// Reuse only a previously owned clean checkout while preserving unchanged input timestamps.
    /// The caller must hold the compilation cache writer lease throughout use of this checkout.
    pub(crate) fn materialize_cached_clean(&self, destination: &Path) -> Result<()> {
        ensure!(self.is_clean(), "cached compilation source requires a clean tracked snapshot");
        let isolated = Workspace { root: destination.to_path_buf() };
        let marker = destination.join(".git/bijux-source-cache.json");
        let origin = self.source.root.canonicalize()?;
        match destination.symlink_metadata() {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                self.materialize(destination)?;
            }
            Err(error) => return Err(error).context("inspect cached source owner"),
            Ok(metadata) => {
                ensure!(
                    metadata.is_dir() && !metadata.file_type().is_symlink(),
                    "cached source owner must be a directory"
                );
                ensure!(
                    !destination.join(".git").symlink_metadata()?.file_type().is_symlink(),
                    "cached Git metadata must not be redirected"
                );
                let ownership: serde_json::Value = serde_json::from_slice(
                    &bijux_dna_infra::read_to_end_bounded(&marker, 16 * 1024)?,
                )?;
                ensure!(
                    ownership["schema_version"] == "bijux.ci.source_cache.v1"
                        && ownership["origin"] == serde_json::json!(origin),
                    "cached source belongs to a different authority"
                );
                let current = TrackedSourceSnapshot::capture(&isolated)?;
                ensure!(
                    current.is_clean() && ownership["commit"] == current.commit(),
                    "cached compilation inputs changed outside their owner"
                );
                ensure!(
                    git(&isolated, &["ls-files", "--others", "--exclude-standard", "-z"])?
                        .is_empty(),
                    "cached source contains untracked input"
                );
                if current.commit() != self.commit() {
                    git(&isolated, &["checkout", "--quiet", "--detach", self.commit()])?;
                }
            }
        }
        bijux_dna_infra::atomic_write_json(
            &marker,
            &serde_json::json!({"schema_version":"bijux.ci.source_cache.v1","origin":origin,"commit":self.commit()}),
        )?;
        Ok(())
    }

    /// Materialize into a new caller-owned scratch directory. Existing paths are never removed.
    pub(crate) fn materialize(&self, destination: &Path) -> Result<()> {
        ensure!(
            destination
                .symlink_metadata()
                .is_err_and(|error| { error.kind() == std::io::ErrorKind::NotFound }),
            "source snapshot destination must not exist: {}",
            destination.display()
        );
        let destination = destination.to_str().context("snapshot destination is not UTF-8")?;
        let source = self.source.root.to_str().context("snapshot source is not UTF-8")?;
        git(
            &self.source,
            &["clone", "--shared", "--quiet", "--no-checkout", "--", source, destination],
        )?;
        let isolated = Workspace { root: Path::new(destination).to_path_buf() };
        git(&isolated, &["checkout", "--quiet", "--detach", &self.commit])?;
        if !self.patch.is_empty() {
            // Git applies binary changes, deletions, link targets and executable modes together.
            // Keep the captured patch in clone metadata, outside tracked input and run artifacts.
            let patch_path = isolated.path(".git/bijux-tracked-source.patch");
            bijux_dna_infra::write_bytes(&patch_path, &self.patch)
                .context("persist captured source patch")?;
            let result = git(
                &isolated,
                &["apply", "--binary", "--whitespace=nowarn", ".git/bijux-tracked-source.patch"],
            );
            bijux_dna_infra::remove_file(&patch_path).context("remove applied source patch")?;
            result?;
        }
        bijux_dna_infra::ensure_dir(isolated.path("artifacts"))?;
        Ok(())
    }
}

fn git(workspace: &Workspace, args: &[&str]) -> Result<Vec<u8>> {
    let command = std::iter::once("git").chain(args.iter().copied()).collect::<Vec<_>>();
    let output = ProcessRunner::new(workspace).run(&command)?;
    ensure!(
        output.status.success(),
        "source snapshot git {}: {}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(output.stdout)
}

#[cfg(test)]
mod tests {
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
        std::fs::set_permissions(
            source.path("modified.txt"),
            std::fs::Permissions::from_mode(0o755),
        )?;
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
    #[test]
    fn cached_clean_snapshot_preserves_unchanged_mtimes_and_advances_changed_inputs() -> Result<()>
    {
        let (root, source) = source_fixture()?;
        let snapshot = TrackedSourceSnapshot::capture(&source)?;
        let cache = root.path().join("cache");
        snapshot.materialize_cached_clean(&cache)?;
        let unchanged = cache.join("unchanged.txt");
        let before = std::fs::metadata(&unchanged)?.modified()?;
        snapshot.materialize_cached_clean(&cache)?;
        assert_eq!(std::fs::metadata(&unchanged)?.modified()?, before);
        std::fs::write(source.path("modified.txt"), b"advanced input with a new length")?;
        git(&source, &["commit", "--quiet", "-am", "fixture: advance cached source"])?;
        TrackedSourceSnapshot::capture(&source)?.materialize_cached_clean(&cache)?;
        assert_eq!(std::fs::read(cache.join("modified.txt"))?, b"advanced input with a new length");
        assert_eq!(std::fs::metadata(&unchanged)?.modified()?, before);
        std::fs::write(cache.join("modified.txt"), b"unexpected edit")?;
        assert!(TrackedSourceSnapshot::capture(&source)?.materialize_cached_clean(&cache).is_err());
        assert_eq!(std::fs::read(cache.join("modified.txt"))?, b"unexpected edit");
        Ok(())
    }

    #[test]
    fn cached_source_never_adopts_unowned_or_dirty_input() -> Result<()> {
        let (root, source) = source_fixture()?;
        let snapshot = TrackedSourceSnapshot::capture(&source)?;
        let unowned = root.path().join("unowned");
        snapshot.materialize(&unowned)?;
        assert!(snapshot.materialize_cached_clean(&unowned).is_err());
        std::fs::write(source.path("modified.txt"), b"dirty input")?;
        let dirty = TrackedSourceSnapshot::capture(&source)?;
        assert!(dirty.materialize_cached_clean(&root.path().join("dirty-cache")).is_err());
        dirty.materialize(&root.path().join("dirty-run"))?;
        assert_eq!(std::fs::read(root.path().join("dirty-run/modified.txt"))?, b"dirty input");
        Ok(())
    }
}
