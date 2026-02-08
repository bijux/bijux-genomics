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
fn policy__boundaries__docs_spine_contract__docs_spine_snapshot() {
    let mut lines = Vec::new();
    for crate_root in crate_roots() {
        let docs = crate_root.join("docs");
        let name = crate_root.file_name().unwrap().to_string_lossy();
        let index = docs.join("INDEX.md");
        let data = read_doc(&index);
        lines.push(format!("[{}]", name));
        lines.push(data);
    }
    let name = bijux_dna_testkit::snapshot_name("snapshots", "crate_docs_spine_contract");
    let mut settings = insta::Settings::new();
    let snapshot_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("snapshots");
    settings.set_snapshot_path(snapshot_root);
    settings.set_prepend_module_to_snapshot(false);
    settings.bind(|| {
        insta::assert_snapshot!(
            name,
            bijux_dna_testkit::snapshot_normalize_text(&lines.join("\n"))
        );
    });
}
