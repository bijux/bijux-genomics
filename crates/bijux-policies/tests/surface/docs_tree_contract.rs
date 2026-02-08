#![allow(non_snake_case)]
use std::path::PathBuf;

use walkdir::WalkDir;

use crate::support::fs::{crate_roots, workspace_root};

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

#[test]
fn policy__surface__docs_tree_contract__docs_tree_contract_snapshot() {
    let docs = workspace_root().join("docs");
    let files = list_files(&docs);
    insta::assert_snapshot!("docs_tree_contract", files.join("\\n"));
}

#[test]
fn policy__surface__docs_tree_contract__crate_docs_tree_contract_snapshot() {
    let mut lines = Vec::new();
    for crate_root in crate_roots() {
        let docs = crate_root.join("docs");
        let name = crate_root.file_name().unwrap().to_string_lossy();
        let files = list_files(&docs);
        lines.push(format!("[{}]", name));
        lines.extend(files.into_iter());
    }
    insta::assert_snapshot!("crate_docs_tree_contract", lines.join(\"\\n\"));
}
