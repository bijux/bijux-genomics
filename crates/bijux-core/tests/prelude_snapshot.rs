#[test]
fn prelude_exports_snapshot() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/prelude/mod.rs");
    let content =
        std::fs::read_to_string(&path).unwrap_or_else(|err| panic!("read prelude: {err}"));
    let snapshot = content
        .lines()
        .filter(|line| line.trim_start().starts_with("pub use"))
        .collect::<Vec<_>>()
        .join("\n");
    let expected_path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/prelude_exports.txt");
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("read prelude fixture: {err}"));
    assert_eq!(
        snapshot, expected,
        "Prelude exports are expected to remain stable; update test if changes are intentional."
    );
}
