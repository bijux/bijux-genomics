use std::fs;
use std::path::Path;

#[test]
fn public_api_snapshot() {
    let lib_rs = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("lib.rs");
    let content = fs::read_to_string(&lib_rs).expect("read lib.rs");
    let mut exports: Vec<String> = content
        .lines()
        .filter_map(|line| line.trim().strip_prefix("pub use "))
        .map(|line| line.trim_end_matches(';').to_string())
        .collect();
    exports.sort();
    let snapshot = exports.join("\n") + "\n";

    let snapshot_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("snapshots")
        .join("public_api.txt");
    let expected = fs::read_to_string(&snapshot_path).expect("read public_api.txt snapshot");
    assert_eq!(snapshot, expected, "public API snapshot must match");
}
