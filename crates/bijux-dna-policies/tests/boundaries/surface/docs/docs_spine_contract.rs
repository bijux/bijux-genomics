#![allow(non_snake_case)]
#[path = "../../../support/fs.rs"]
mod support;

use std::path::Path;

use support::crate_roots;

fn read_doc(path: &Path) -> String {
    bijux_dna_testkit::read_policy_text(path)
}

/// Checks crate docs index files without snapshotting the full workspace docs tree.
#[test]
fn policy__boundaries__docs_spine_contract__docs_spine_snapshot() {
    let mut missing = Vec::new();
    for crate_root in crate_roots() {
        let docs = crate_root.join("docs");
        let name = crate_root.file_name().unwrap().to_string_lossy();
        let index = docs.join("INDEX.md");
        if !index.exists() {
            continue;
        }
        let data = read_doc(&index);
        if !data.lines().any(|line| line.starts_with("# ")) {
            missing.push(format!("{name}: docs/INDEX.md missing H1"));
        }
    }
    bijux_dna_policies::policy_assert!(
        missing.is_empty(),
        "crate docs indexes must include an H1:\n{}",
        missing.join("\n")
    );
}
