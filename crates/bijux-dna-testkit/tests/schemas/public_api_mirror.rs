use std::collections::BTreeSet;
use std::path::Path;

#[test]
fn public_api_namespace_mirrors_root_reexports() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let lib_rs = std::fs::read_to_string(root.join("src/lib.rs"))
        .unwrap_or_else(|err| panic!("read src/lib.rs: {err}"));
    let surface_rs = std::fs::read_to_string(root.join("src/public_api/surface.rs"))
        .unwrap_or_else(|err| panic!("read src/public_api/surface.rs: {err}"));

    assert_eq!(
        collect_pub_uses(&lib_rs),
        collect_pub_uses(&surface_rs),
        "src/public_api/surface.rs must mirror root re-exports"
    );
}

fn collect_pub_uses(content: &str) -> BTreeSet<String> {
    let mut exports = BTreeSet::new();
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
            exports.insert(normalize_export(&buffer));
        }
    }

    exports
}

fn normalize_export(raw: &str) -> String {
    let mut normalized = raw.trim_end_matches(';').split_whitespace().collect::<Vec<_>>().join(" ");
    normalized = normalized.replace("crate::", "");
    normalized = normalized.replace("{ ", "{");
    normalized = normalized.replace(" }", "}");
    normalized = normalized.replace(", }", "}");
    normalized = normalized.replace(",}", "}");
    normalized
}
