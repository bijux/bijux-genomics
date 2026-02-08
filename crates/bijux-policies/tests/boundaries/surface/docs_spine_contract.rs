#![allow(non_snake_case)]
#[path = "../../support/fs.rs"]
mod support;

use std::path::PathBuf;

use support::crate_roots;

fn read_doc(path: &PathBuf) -> String {
    std::fs::read_to_string(path).expect("read doc")
}

/// Snapshot locks crate docs spine contents to prevent drift.
#[test]
fn policy__surface__docs_spine_contract__docs_spine_snapshot() {
    let mut lines = Vec::new();
    for crate_root in crate_roots() {
        let docs = crate_root.join("docs");
        let name = crate_root.file_name().unwrap().to_string_lossy();
        let index = docs.join("INDEX.md");
        let data = read_doc(&index);
        lines.push(format!("[{}]", name));
        lines.push(data);
    }
    let name = bijux_testkit::snapshot_name("snapshots", "crate_docs_spine_contract");
    insta::assert_snapshot!(name, lines.join("\n"));
}
