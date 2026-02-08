#![allow(non_snake_case)]
use std::collections::BTreeSet;

use crate::support::crate_roots;

const REQUIRED_DOCS: &[&str] = &[
    "INDEX.md",
    "ARCHITECTURE.md",
    "SCOPE.md",
    "TESTS.md",
    "CHANGE_RULES.md",
    "EFFECTS.md",
];

const ALLOW_MISSING: &[(&str, &[&str])] = &[("bijux-engine", &["ARCHITECTURE.md"])];

fn allowed_missing_for(crate_name: &str) -> BTreeSet<&'static str> {
    ALLOW_MISSING
        .iter()
        .find(|(name, _)| *name == crate_name)
        .map(|(_, missing)| missing.iter().copied().collect())
        .unwrap_or_default()
}

#[test]
fn policy__surface__docs_spine_contract__crate_docs_spine_contract_snapshot() {
    let mut lines = Vec::new();

    for crate_root in crate_roots() {
        let crate_name = crate_root.file_name().unwrap().to_string_lossy();
        let docs_dir = crate_root.join("docs");
        if !docs_dir.exists() {
            continue;
        }
        let mut present = BTreeSet::new();
        for entry in std::fs::read_dir(&docs_dir).expect("read docs dir") {
            let entry = entry.expect("read entry");
            if entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
                present.insert(entry.file_name().to_string_lossy().to_string());
            }
        }

        let allowed_missing = allowed_missing_for(&crate_name);
        let mut expected = BTreeSet::new();
        for doc in REQUIRED_DOCS {
            if !allowed_missing.contains(doc) {
                expected.insert(doc.to_string());
            }
        }

        let missing: Vec<_> = expected
            .iter()
            .filter(|doc| !present.contains(*doc))
            .cloned()
            .collect();
        if !missing.is_empty() {
            bijux_policies::policy_panic!(
                "crate docs spine missing required docs in {}: {:?}",
                crate_root.display(),
                missing
            );
        }

        lines.push(format!("[{crate_name}]"));
        for doc in expected {
            lines.push(doc);
        }
    }

    insta::assert_snapshot!("crate_docs_spine_contract", lines.join("\n"));
}
