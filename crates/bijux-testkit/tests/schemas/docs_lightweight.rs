use std::fs;
use std::path::PathBuf;

#[test]
fn testkit_docs_emphasize_lightweight() {
    let doc = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("docs")
        .join("INDEX.md");
    let content = fs::read_to_string(&doc).expect("read INDEX.md");
    assert!(
        content.contains("shared fixtures"),
        "INDEX.md must describe shared fixtures only"
    );
}
