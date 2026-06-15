#![allow(clippy::expect_used)]

use std::fs;
use std::path::PathBuf;

#[test]
fn explainability_docs_reference_tests() {
    let doc = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs").join("EXPLAIN_OUTPUT.md");
    let content = fs::read_to_string(&doc).expect("read EXPLAIN_OUTPUT.md");
    assert!(
        content.contains("tests/contracts/explain/explainability.rs"),
        "EXPLAIN_OUTPUT.md must reference tests/contracts/explain/explainability.rs"
    );
}
