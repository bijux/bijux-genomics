#![allow(non_snake_case)]
#![allow(clippy::expect_used, clippy::unwrap_used)]

#[path = "../../../support/fs.rs"]
mod support;

use std::path::Path;

const REQUIRED_DOCS: &[&str] = &["ARCHITECTURE.md"];
fn has_uppercase_name(path: &Path) -> bool {
    path.file_stem().and_then(|name| name.to_str()).is_some_and(|name| name == name.to_uppercase())
}

#[test]
fn policy__boundaries__docs_required_policy__crates_require_architecture_docs() {
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

    bijux_dna_policies::policy_assert!(
        missing.is_empty(),
        "crates must include ARCHITECTURE.md in docs/.\n\
Fix by adding the docs under crates/<crate>/docs (UPPERCASE).\n\
See crates/bijux-dna-policies/docs/INDEX.md for the current documentation spine.\n\
Missing:\n{}",
        missing.join("\n")
    );
}

#[test]
fn policy__boundaries__docs_required_policy__crate_docs_use_uppercase_names() {
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
            if !has_uppercase_name(&path) {
                offenders.push(path.display().to_string());
            }
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "crate docs must use uppercase names.\n\
Fix by renaming docs to UPPERCASE.\n\
See docs/40-policies/STYLE.md for naming rules.\n\
Offenders:\n{}",
        offenders.join("\n")
    );
}
