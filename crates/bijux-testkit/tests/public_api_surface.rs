use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

#[test]
fn public_surface_is_deliberate() {
    let lib_rs = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("lib.rs");
    let content = fs::read_to_string(&lib_rs).expect("read lib.rs");
    let mut exports = BTreeSet::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("pub use ") {
            exports.insert(rest.trim_end_matches(';').to_string());
        }
    }
    let expected: BTreeSet<String> = ["snapshots::stable_json".to_string()].into_iter().collect();
    assert_eq!(exports, expected, "public surface must stay minimal");
}
