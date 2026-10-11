use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use anyhow::{ensure, Result};
use walkdir::WalkDir;

use crate::commands::command_support::run_command;
use crate::runtime::workspace::Workspace;

/// Science source and generated metadata belong in Git; acquired evidence does not.
pub(super) fn science_tree_offenders(workspace: &Workspace) -> Result<Vec<String>> {
    let output = run_command(workspace, "git", &["ls-files", "-z", "--", "science"])?;
    ensure!(output.status.success(), "cannot read science index for archive boundary check");
    let tracked = std::str::from_utf8(&output.stdout)?
        .split('\0')
        .filter(|path| !path.is_empty())
        .map(PathBuf::from)
        .collect::<BTreeSet<_>>();
    let directories = tracked
        .iter()
        .flat_map(|path| path.ancestors().skip(1).map(Path::to_path_buf))
        .collect::<BTreeSet<_>>();
    let science = workspace.path("science");
    if !science.exists() {
        return Ok(Vec::new());
    }
    let mut offenders = Vec::new();
    let mut entries = WalkDir::new(science).follow_links(false).into_iter();
    while let Some(entry) = entries.next() {
        let entry = entry?;
        let relative = workspace.rel(entry.path());
        let kind = entry.file_type();
        if kind.is_dir() && directories.contains(relative) {
            continue;
        }
        let metadata = relative == Path::new("science/docs/.gitignore")
            || matches!(
                relative.extension().and_then(|extension| extension.to_str()),
                Some("md" | "tsv" | "json" | "yaml")
            );
        if kind.is_file() && metadata && tracked.contains(relative) {
            continue;
        }
        offenders.push(format!(
            "{}: science permits indexed source and metadata only; preserve downloaded evidence in the external science archive",
            relative.display()
        ));
        if kind.is_dir() {
            entries.skip_current_dir();
        }
    }
    offenders.sort();
    Ok(offenders)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Result<(tempfile::TempDir, Workspace)> {
        let root = tempfile::tempdir()?;
        let workspace = Workspace { root: root.path().to_path_buf() };
        ensure!(run_command(&workspace, "git", &["init", "-q"])?.status.success());
        std::fs::create_dir_all(workspace.path("science/docs"))?;
        std::fs::write(workspace.path("science/docs/README.md"), "evidence locators\n")?;
        std::fs::write(workspace.path("science/docs/.gitignore"), "downloads/\n")?;
        ensure!(run_command(&workspace, "git", &["add", "science"])?.status.success());
        Ok((root, workspace))
    }

    #[test]
    fn detects_ignored_downloads_and_indexed_payloads_without_removing_evidence() -> Result<()> {
        let (_root, workspace) = fixture()?;
        assert!(science_tree_offenders(&workspace)?.is_empty());
        std::fs::create_dir_all(workspace.path("science/docs/downloads/repo/.git"))?;
        std::fs::write(workspace.path("science/docs/downloads/paper.pdf"), b"evidence")?;
        std::fs::write(workspace.path("science/docs/source.tar.gz"), b"archive")?;
        ensure!(run_command(&workspace, "git", &["add", "science/docs/source.tar.gz"])?
            .status
            .success());
        let offenders = science_tree_offenders(&workspace)?;
        assert_eq!(offenders.len(), 2, "{offenders:?}");
        assert!(offenders.iter().any(|item| item.starts_with("science/docs/downloads:")));
        assert!(offenders.iter().any(|item| item.starts_with("science/docs/source.tar.gz:")));
        assert_eq!(std::fs::read(workspace.path("science/docs/downloads/paper.pdf"))?, b"evidence");
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn rejects_archive_symlinks_even_when_indexed() -> Result<()> {
        let (_root, workspace) = fixture()?;
        std::os::unix::fs::symlink("/missing/external/archive", workspace.path("science/archive"))?;
        ensure!(run_command(&workspace, "git", &["add", "science/archive"])?.status.success());
        let offenders = science_tree_offenders(&workspace)?;
        assert_eq!(offenders.len(), 1);
        assert!(offenders[0].starts_with("science/archive:"));
        Ok(())
    }
}
