use std::path::{Path, PathBuf};

use walkdir::WalkDir;

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

#[test]
fn policy__boundaries__empty_tests_dirs__no_empty_tests_dirs() {
    let root = workspace_root();
    let mut offenders = Vec::new();
    for entry in WalkDir::new(root.join("crates"))
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_dir())
    {
        if entry.path().components().any(|c| c.as_os_str() == "tests") {
            let mut has_files = false;
            if let Ok(mut entries) = std::fs::read_dir(entry.path()) {
                has_files = entries.any(|child| child.ok().map(|c| c.file_type().ok().map(|t| t.is_dir() || t.is_file()).unwrap_or(false)).unwrap_or(false));
            }
            if !has_files {
                offenders.push(entry.path().display().to_string());
            }
        }
    }
    bijux_policies::policy_assert!(
        offenders.is_empty(),
        "empty tests/ directories are forbidden: {offenders:?}"
    );
}
