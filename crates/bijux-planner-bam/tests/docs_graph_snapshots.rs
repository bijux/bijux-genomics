use std::fs;
use std::path::PathBuf;

#[test]
fn graph_snapshots_are_documented() {
    let doc = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("docs")
        .join("EXPLAIN_OUTPUT.md");
    let content = fs::read_to_string(&doc).expect("read EXPLAIN_OUTPUT.md");
    assert!(
        content.contains("tests/graph_snapshots.rs"),
        "EXPLAIN_OUTPUT.md must reference tests/graph_snapshots.rs"
    );
}
