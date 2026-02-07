use std::fs;
use std::path::PathBuf;

#[test]
fn public_api_has_doc_examples() {
    let doc = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("docs")
        .join("DETERMINISM.md");
    let content = fs::read_to_string(&doc).expect("read DETERMINISM.md");

    for func in ["score_suite", "classify_gate"] {
        assert!(
            content.contains(func),
            "DETERMINISM.md must include example for {}",
            func
        );
    }
}
