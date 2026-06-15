#![allow(non_snake_case)]
#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::path::Path;

use walkdir::WalkDir;

#[path = "../../../support/fs.rs"]
mod support;

#[test]
fn policy__boundaries__coverage_cfg_policy__cfg_coverage_is_banned_in_src() {
    let root = support::workspace_root();
    let crates_root = root.join("crates");
    let mut offenders = Vec::new();

    for entry in WalkDir::new(&crates_root) {
        let entry = entry.expect("walk crates");
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        if !is_src_path(entry.path()) {
            continue;
        }
        let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
        if content.contains("cfg(coverage)") || content.contains("#[cfg(coverage)]") {
            offenders.push(rel_path(&root, entry.path()));
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "cfg(coverage) is forbidden in production src paths: {offenders:?}"
    );
}

fn is_src_path(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    path_str.contains("/src/")
        && !path_str.contains("/tests/")
        && !path_str.contains("/benches/")
        && !path_str.contains("/dev-tools/")
}

fn rel_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root).unwrap_or(path).to_string_lossy().to_string()
}
