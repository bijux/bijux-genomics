#![allow(non_snake_case)]

use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

#[test]
fn policy__boundaries__no_repo_tree_snapshots__forbid_tree_contract_snapshots() {
    let root = workspace_root();
    let snapshots_dir = root.join("crates/bijux-policies/tests/snapshots");
    let mut offenders = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&snapshots_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("snap") {
                continue;
            }
            let name = path
                .file_name()
                .and_then(|f| f.to_str())
                .unwrap_or_default();
            if name.contains("tree_contract") {
                offenders.push(path.display().to_string());
            }
        }
    }

    bijux_policies::policy_assert!(
        offenders.is_empty(),
        "repo tree snapshots are not allowed; keep only API surface or schema snapshots:\n{}",
        offenders.join("\n")
    );
}
