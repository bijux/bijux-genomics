use std::fs;
use std::path::PathBuf;

#[test]
fn reference_matrix_is_documented() {
    let doc = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("docs")
        .join("ENV_REFERENCE.md");
    let content =
        fs::read_to_string(&doc).unwrap_or_else(|err| panic!("read ENV_REFERENCE.md: {err}"));
    assert!(
        content.contains("tests/matrix/reference_matrix.rs"),
        "ENV_REFERENCE.md must reference tests/matrix/reference_matrix.rs"
    );
}
