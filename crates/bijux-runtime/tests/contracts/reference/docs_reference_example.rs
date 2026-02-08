use std::fs;
use std::path::PathBuf;

#[test]
fn reference_example_is_documented() {
    let doc = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("docs")
        .join("RUNTIME_CONTRACT.md");
    let content = fs::read_to_string(&doc)
        .unwrap_or_else(|err| panic!("read RUNTIME_CONTRACT.md at {}: {err}", doc.display()));

    assert!(
        content.contains("tests/reference/reference_example.rs"),
        "docs must reference tests/reference/reference_example.rs"
    );
}
