#![allow(non_snake_case)]
#![allow(non_snake_case)]
#[path = "../../support/fs.rs"]
mod support;

use std::path::PathBuf;

use walkdir::WalkDir;

use support::crate_roots;

fn list_files(root: &PathBuf) -> Vec<String> {
    let mut files = Vec::new();
    for entry in WalkDir::new(root) {
        let entry = entry.expect("walk");
        if entry.file_type().is_file() {
            let rel = entry
                .path()
                .strip_prefix(root)
                .unwrap()
                .to_string_lossy()
                .to_string();
            files.push(rel);
        }
    }
    files.sort();
    files
}

/// Snapshot locks crate tree layout to prevent drift.
#[test]
fn policy__boundaries__crate_tree_contract__crate_tree_contract_snapshot() {
    let mut lines = Vec::new();
    for crate_root in crate_roots() {
        let name = crate_root.file_name().unwrap().to_string_lossy();
        let files = list_files(&crate_root);
        lines.push(format!("[{}]", name));
        lines.extend(files.into_iter());
    }
    let name = bijux_testkit::snapshot_name("snapshots", "crate_tree_contract");
    let mut settings = insta::Settings::new();
    settings.set_snapshot_path(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/snapshots"),
    );
    settings.set_prepend_module_to_snapshot(false);
    settings.bind(|| {
        insta::assert_snapshot!(name, lines.join("\n"));
    });
}
