use std::fs;
use std::path::PathBuf;

#[test]
fn reference_matrix_is_documented() {
    let doc = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("docs")
        .join("ENV_REFERENCE.md");
    let content = fs::read_to_string(&doc).expect("read ENV_REFERENCE.md");
    assert!(
        content.contains("tests/reference_matrix.rs"),
        "ENV_REFERENCE.md must reference tests/reference_matrix.rs"
    );
}
