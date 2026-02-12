use std::fs;
use std::path::Path;

#[test]
fn public_api_snapshot() {
    let lib_rs = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("lib.rs");
    let content =
        fs::read_to_string(&lib_rs).unwrap_or_else(|err| panic!("read lib.rs failed: {err}"));
    let mut exports = collect_pub_uses(&content);
    exports.sort();
    let snapshot = exports.join("\n") + "\n";

    let snapshot_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("snapshots")
        .join("bijux-dna-testkit__schemas__public_api.txt");
    let expected = fs::read_to_string(&snapshot_path)
        .unwrap_or_else(|err| panic!("read public_api.txt snapshot failed: {err}"));
    assert_eq!(snapshot, expected, "public API snapshot must match");
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
