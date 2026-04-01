#[test]
fn prelude_exports_snapshot() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/prelude/mod.rs");
    let content =
        std::fs::read_to_string(&path).unwrap_or_else(|err| panic!("read prelude: {err}"));
    let mut snapshot = content
        .lines()
        .filter(|line| line.trim_start().starts_with("pub use"))
        .collect::<Vec<_>>()
        .into_iter()
        .map(str::trim)
        .collect::<Vec<_>>();
    snapshot.sort_unstable();
    let expected_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("prelude_exports")
        .join("default")
        .join("prelude_exports.txt");
    let expected_text = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("read prelude fixture: {err}"));
    let mut expected = expected_text.lines().map(str::trim).collect::<Vec<_>>();
    expected.sort_unstable();
    assert_eq!(
        snapshot,
        expected,
        "Prelude exports are expected to remain stable; ordering changes alone should not fail the contract."
    );
}
