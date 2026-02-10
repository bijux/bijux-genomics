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
    for export in collect_pub_uses(&content) {
        exports.insert(export);
    }
    let expected: BTreeSet<String> = [
        "determinism::{assert_json_stable, assert_stable_ordering, strip_timestamp_fields}"
            .to_string(),
        "fixtures::{assert_json_schema_like, load_fixture_json, load_fixture_text}".to_string(),
        "snapshots::{install_snapshot_env, sanitize_snapshot_json, sanitize_snapshot_text, snapshot_name, snapshot_normalize_json, snapshot_normalize_text, stable_json}"
            .to_string(),
        "temp::{resolve_under, temp_path_for, tempdir_for}".to_string(),
    ]
    .into_iter()
    .collect();
    assert_eq!(exports, expected, "public surface must stay minimal");
}

fn collect_pub_uses(content: &str) -> Vec<String> {
    let mut exports = Vec::new();
    let mut lines = content.lines().peekable();
    while let Some(line) = lines.next() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("pub use ") {
            let mut buffer = rest.to_string();
            while !buffer.trim_end().ends_with(';') {
                if let Some(next) = lines.next() {
                    buffer.push(' ');
                    buffer.push_str(next.trim());
                } else {
                    break;
                }
            }
            exports.push(normalize_export(&buffer));
        }
    }
    exports
}

fn normalize_export(raw: &str) -> String {
    let mut normalized = raw
        .trim_end_matches(';')
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    normalized = normalized.replace("{ ", "{");
    normalized = normalized.replace(" }", "}");
    normalized = normalized.replace(", }", "}");
    normalized = normalized.replace(",}", "}");
    normalized
}
