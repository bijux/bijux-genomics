use anyhow::{ensure, Context, Result};
use std::path::Path;
pub(super) fn collect_raw_fixture_paths(repo_root: &Path, root: &Path) -> Result<Vec<String>> {
    let mut paths = Vec::new();
    let mut directories = vec![root.to_path_buf()];
    while let Some(directory) = directories.pop() {
        ensure!(
            !std::fs::symlink_metadata(&directory)?.file_type().is_symlink(),
            "parser fixture traversal refuses symbolic links"
        );
        for entry in std::fs::read_dir(&directory)
            .with_context(|| format!("read {}", directory.display()))?
        {
            let entry =
                entry.with_context(|| format!("read entry under {}", directory.display()))?;
            let kind = entry.file_type()?;
            let path = entry.path();
            ensure!(!kind.is_symlink(), "parser fixture traversal refuses symbolic links");
            if kind.is_dir() {
                directories.push(path);
            } else if kind.is_file()
                && path
                    .file_name()
                    .and_then(|value| value.to_str())
                    .is_some_and(|name| name.starts_with("raw."))
            {
                paths.push(
                    path.strip_prefix(repo_root)
                        .unwrap_or(&path)
                        .to_string_lossy()
                        .replace('\\', "/"),
                );
            }
        }
    }
    paths.sort();
    Ok(paths)
}
