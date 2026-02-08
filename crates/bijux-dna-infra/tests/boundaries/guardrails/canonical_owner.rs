use std::fs;
use std::path::PathBuf;

#[test]
fn paths_doc_points_to_core() {
    let doc = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("docs")
        .join("PATHS.md");
    let content = fs::read_to_string(&doc)
        .unwrap_or_else(|err| panic!("read PATHS.md at {}: {err}", doc.display()));
    assert!(
        content.to_lowercase().contains("bijux-dna-core"),
        "PATHS.md must point to bijux-dna-core as canonical owner"
    );
}
