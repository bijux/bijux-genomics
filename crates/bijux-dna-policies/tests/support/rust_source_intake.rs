use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub(super) fn collect_rs_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let kind = std::fs::symlink_metadata(dir)
        .with_context(|| format!("inspect required Rust source {}", dir.display()))?
        .file_type();
    anyhow::ensure!(
        kind.is_dir() && !kind.is_symlink(),
        "required Rust source must be an unredirected directory: {}",
        dir.display()
    );
    let mut files = Vec::new();
    for entry in WalkDir::new(dir) {
        let entry =
            entry.with_context(|| format!("walk required Rust source {}", dir.display()))?;
        anyhow::ensure!(
            !entry.file_type().is_symlink(),
            "required Rust source must not redirect: {}",
            entry.path().display()
        );
        if entry.file_type().is_dir()
            || entry.path().extension().and_then(|extension| extension.to_str()) != Some("rs")
        {
            continue;
        }
        anyhow::ensure!(
            entry.file_type().is_file(),
            "required Rust source must be a regular file: {}",
            entry.path().display()
        );
        files.push(entry.into_path());
    }
    files.sort();
    anyhow::ensure!(
        !files.is_empty(),
        "required Rust source contains no Rust files: {}",
        dir.display()
    );
    Ok(files)
}
