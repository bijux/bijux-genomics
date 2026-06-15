#![allow(clippy::expect_used)]

use std::path::PathBuf;

#[test]
fn symmetry_rules_match_docs() {
    let doc = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs").join("STAGE_CONTRACTS.md");
    let content = std::fs::read_to_string(&doc).expect("read STAGE_CONTRACTS.md");
    let content = content.to_lowercase();
    assert!(content.contains("symmetry"), "docs/STAGE_CONTRACTS.md must mention symmetry rules");
    assert!(
        content.contains("contract level"),
        "docs/STAGE_CONTRACTS.md must state symmetry is contract-level"
    );
}
