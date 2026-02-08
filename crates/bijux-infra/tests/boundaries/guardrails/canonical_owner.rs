use std::fs;
use std::path::PathBuf;

#[test]
fn paths_doc_points_to_core() {
    let doc = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("docs")
        .join("PATHS.md");
    let content = fs::read_to_string(&doc).expect("read PATHS.md");
    assert!(
        content.to_lowercase().contains("bijux-core"),
        "PATHS.md must point to bijux-core as canonical owner"
    );
}
