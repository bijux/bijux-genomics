use std::path::PathBuf;

use regex::Regex;
use walkdir::WalkDir;

const DISALLOWED_PREFIXES: &[&str] = &["normalize_", "sanitize_", "clean_", "helper_", "util_"];
const ALLOWLIST: &[&str] = &["normalize_run_base_dir"];

#[test]
fn infra_does_not_expose_generic_helpers() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let re = Regex::new(r"pub\s+fn\s+([a-zA-Z0-9_]+)").expect("compile regex");
    let mut offenders = Vec::new();

    for entry in WalkDir::new(&root) {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }
        let content = std::fs::read_to_string(entry.path()).expect("read source");
        for cap in re.captures_iter(&content) {
            let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            if ALLOWLIST.contains(&name) {
                continue;
            }
            if DISALLOWED_PREFIXES
                .iter()
                .any(|prefix| name.starts_with(prefix))
            {
                offenders.push(format!("{} ({})", entry.path().display(), name));
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "infra must not expose generic helper-style APIs.\nOffenders:\n{}",
        offenders.join("\n")
    );
}
