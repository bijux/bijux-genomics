use anyhow::Result;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub(crate) fn collect_rs_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = WalkDir::new(root)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.file_type().is_file()
                && entry.path().extension().and_then(|suffix| suffix.to_str()) == Some("rs")
        })
        .map(walkdir::DirEntry::into_path)
        .collect::<Vec<_>>();
    files.sort();
    Ok(files)
}
