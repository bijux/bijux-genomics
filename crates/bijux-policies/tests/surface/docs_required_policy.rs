#[path = "../support/fs.rs"]
mod support;

use std::path::Path;

const REQUIRED_DOCS: &[&str] = &["SCOPE.md", "ARCHITECTURE.md"];
const README_REQUIRED: &[(&str, &str)] = &[("bijux-cli", "public CLI entrypoint")];

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
        let docs_root = crate_root.join("docs");
        if !docs_root.exists() {
            missing.push(format!("{} missing docs/ directory", crate_root.display()));
            continue;
        }
        for doc in REQUIRED_DOCS {
            let doc_path = docs_root.join(doc);
            if !doc_path.exists() {
                missing.push(format!("{} missing {}", crate_root.display(), doc));
            }
        }
    }

    assert!(
        missing.is_empty(),
        "crates must include SCOPE.md and ARCHITECTURE.md.\n\
Fix by adding the docs at crate root (UPPERCASE). If README.md is required, add to allowlist with reason.\n\
See STYLE.md for documentation spine.\n\
Missing:\n{}",
        missing.join("\n")
    );
}

#[test]
fn crate_docs_use_uppercase_names() {
    let mut offenders = Vec::new();
    for crate_root in support::crate_roots() {
        let docs_root = crate_root.join("docs");
        if !docs_root.exists() {
            continue;
        }
        for entry in std::fs::read_dir(&docs_root).expect("read docs root") {
            let entry = entry.expect("read entry");
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
                continue;
            }
            if path.file_name().and_then(|name| name.to_str()) == Some("README.md") {
                continue;
            }
            if !has_uppercase_name(&path) {
                offenders.push(path.display().to_string());
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "crate docs must use uppercase names.\n\
Fix by renaming docs to UPPERCASE (README.md allowed when allowlisted).\n\
See STYLE.md for naming rules.\n\
Offenders:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn crates_require_readme_when_listed() {
    let mut missing = Vec::new();
    for (crate_name, reason) in README_REQUIRED {
        let crate_root = support::crate_root(crate_name);
        let doc_path = crate_root.join("docs").join("README.md");
        if !doc_path.exists() {
            missing.push(format!(
                "{} missing docs/README.md (reason: {reason})",
                crate_root.display()
            ));
        }
    }

    assert!(
        missing.is_empty(),
        "crates listed in README_REQUIRED must include README.md.\n\
How to fix: add README.md at crate root or update README_REQUIRED with a reason.\n\
See STYLE.md for documentation spine.\n\
Missing:\n{}",
        missing.join("\n")
    );
}
