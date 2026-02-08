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
fn policy__contracts__snapshot_hygiene__no_absolute_paths_or_hostnames() {
    let root = workspace_root();
    let mut offenders = Vec::new();
    for entry in WalkDir::new(root.join("crates"))
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
    {
        if entry.path().extension().and_then(|s| s.to_str()) != Some("snap") {
            continue;
        }
        let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
        let bad = content.contains("/Users/")
            || content.contains("\\Users\\")
            || content.contains("/tmp/")
            || content.contains("C:\\\\");
        if bad {
            offenders.push(entry.path().display().to_string());
        }
    }
    bijux_policies::policy_assert!(
        offenders.is_empty(),
        "snapshots must not include absolute paths or hostnames: {offenders:?}"
    );
}
