use std::fs;
use std::path::PathBuf;

#[test]
fn reference_example_is_documented() {
    let doc = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("docs")
        .join("RUNTIME_CONTRACT.md");
    let content = fs::read_to_string(&doc).expect("read RUNTIME_CONTRACT.md");

    assert!(
        content.contains("tests/reference_example.rs"),
        "docs must reference tests/reference_example.rs"
    );
}
