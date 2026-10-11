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
            std::fs::write(&patch_path, &self.patch).context("persist captured source patch")?;
            let result = git(
                &isolated,
                &["apply", "--binary", "--whitespace=nowarn", ".git/bijux-tracked-source.patch"],
            );
            std::fs::remove_file(&patch_path).context("remove applied source patch")?;
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
mod tests;
