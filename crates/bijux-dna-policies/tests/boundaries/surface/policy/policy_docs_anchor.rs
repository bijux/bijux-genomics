#![allow(non_snake_case)]
#[path = "../../../support/fs.rs"]
mod support;

use std::collections::BTreeSet;

use walkdir::WalkDir;

fn policy_files() -> BTreeSet<String> {
    let mut files = BTreeSet::new();
    let base = support::workspace_root().join("crates/bijux-dna-policies/tests/boundaries");
    for folder in ["deps", "surface", "data", "tooling"] {
        let root = base.join(folder);
        if !root.exists() {
            continue;
        }
        for entry in WalkDir::new(&root) {
            let entry =
                entry.unwrap_or_else(|err| panic!("walk docs under {}: {err}", root.display()));
            if !entry.file_type().is_file() {
                continue;
            }
            if entry.path().extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            files.insert(entry.file_name().to_string_lossy().to_string());
        }
    }
    files
}

#[test]
fn policy__boundaries__policy_docs_anchor__policy_docs_index_covers_all_policies() {
    let index_path = support::workspace_root().join("docs/INDEX.md");
    let index = support::read_to_string(&index_path);
    let policies = policy_files();
    let mut missing = Vec::new();
    for policy in policies {
        if !index.contains(&policy) {
            missing.push(policy);
        }
    }
    bijux_dna_policies::policy_assert!(
        missing.is_empty(),
        "policy INDEX.md must list every policy test file.\n\
How to fix: add missing filenames to docs/INDEX.md.\n\
Missing:\n{}",
        missing.join("\n")
    );
}
