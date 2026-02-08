#![allow(non_snake_case)]
#![allow(non_snake_case)]
#[path = "../../support/fs.rs"]
mod support;

use std::path::PathBuf;

use walkdir::WalkDir;

use support::{crate_roots, workspace_root};

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

/// Snapshot locks workspace docs tree to prevent drift.
#[test]
fn policy__boundaries__docs_tree_contract__docs_tree_contract_snapshot() {
    let docs = workspace_root().join("docs");
    let files = list_files(&docs);
    let name = bijux_testkit::snapshot_name("snapshots", "docs_tree_contract");
    let mut settings = insta::Settings::new();
    let snapshot_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("snapshots");
    settings.set_snapshot_path(snapshot_root);
    settings.set_prepend_module_to_snapshot(false);
    settings.bind(|| {
        insta::assert_snapshot!(
            name,
            bijux_testkit::snapshot_normalize_text(&files.join("\n"))
        );
    });
}

/// Snapshot locks per-crate docs trees to prevent drift.
#[test]
fn policy__boundaries__docs_tree_contract__crate_docs_tree_contract_snapshot() {
    let mut lines = Vec::new();
    for crate_root in crate_roots() {
        let docs = crate_root.join("docs");
        let name = crate_root.file_name().unwrap().to_string_lossy();
        let files = list_files(&docs);
        lines.push(format!("[{}]", name));
        lines.extend(files.into_iter());
    }
    let name = bijux_testkit::snapshot_name("snapshots", "crate_docs_tree_contract");
    let mut settings = insta::Settings::new();
    let snapshot_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("snapshots");
    settings.set_snapshot_path(snapshot_root);
    settings.set_prepend_module_to_snapshot(false);
    settings.bind(|| {
        insta::assert_snapshot!(
            name,
            bijux_testkit::snapshot_normalize_text(&lines.join("\n"))
        );
    });
}
