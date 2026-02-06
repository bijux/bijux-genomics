#[path = "../support/fs.rs"]
mod support;

use std::path::Path;

const REQUIRED_DOCS: &[&str] = &["SCOPE.md", "ARCHITECTURE.md"];
const README_ALLOWLIST: &[&str] = &[];

fn has_uppercase_name(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.chars().all(|ch| !ch.is_ascii_lowercase()))
        .unwrap_or(false)
}

#[test]
fn crates_require_scope_and_architecture_docs() {
    let mut missing = Vec::new();
    for crate_root in support::crate_roots() {
        for doc in REQUIRED_DOCS {
            let doc_path = crate_root.join(doc);
            if !doc_path.exists() {
                missing.push(format!("{} missing {}", crate_root.display(), doc));
            }
        }
    }

    assert!(
        missing.is_empty(),
        "crates must include SCOPE.md and ARCHITECTURE.md:\n{}",
        missing.join("\n")
    );
}

#[test]
fn crate_docs_use_uppercase_names() {
    let mut offenders = Vec::new();
    for crate_root in support::crate_roots() {
        for entry in std::fs::read_dir(&crate_root).expect("read crate root") {
            let entry = entry.expect("read entry");
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
                continue;
            }
            if path.file_name().and_then(|name| name.to_str()) == Some("README.md") {
                if README_ALLOWLIST.contains(&crate_root.file_name().unwrap().to_string_lossy().as_ref()) {
                    continue;
                }
                continue;
            }
            if !has_uppercase_name(&path) {
                offenders.push(path.display().to_string());
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "crate docs must use uppercase names:\n{}",
        offenders.join("\n")
    );
}
