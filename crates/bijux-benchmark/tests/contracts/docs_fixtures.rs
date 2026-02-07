use std::fs;
use std::path::PathBuf;

#[test]
fn fixtures_are_documented() {
    let doc = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("docs")
        .join("BENCH_FORMAT.md");
    let content = fs::read_to_string(&doc).expect("read BENCH_FORMAT.md");
    assert!(
        content.contains("decision.json"),
        "BENCH_FORMAT.md must describe decision.json"
    );
}
