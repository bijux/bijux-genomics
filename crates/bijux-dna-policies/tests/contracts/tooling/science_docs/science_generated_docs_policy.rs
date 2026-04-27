#![allow(non_snake_case)]
#[path = "../../../support/fs.rs"]
mod support;

use std::collections::BTreeSet;
use std::fs;

fn markdown_link_targets(path: &str) -> BTreeSet<String> {
    let root = support::workspace_root();
    let raw =
        fs::read_to_string(root.join(path)).unwrap_or_else(|err| panic!("read {path}: {err}"));
    let mut targets = BTreeSet::new();
    for line in raw.lines() {
        let mut rest = line;
        while let Some((_, suffix)) = rest.split_once("](") {
            if let Some((target, tail)) = suffix.split_once(')') {
                targets.insert(target.to_string());
                rest = tail;
            } else {
                break;
            }
        }
    }
    targets
}

#[test]
fn policy__contracts__science_generated_docs_policy__generated_root_readme_links_governed_subsurfaces_exactly(
) {
    let expected = BTreeSet::from([
        "../README.md".to_string(),
        "../specs/evidence/README.md".to_string(),
        "current/README.md".to_string(),
        "indexes/README.md".to_string(),
    ]);
    let documented = markdown_link_targets("science/generated/README.md");
    assert_eq!(
        expected, documented,
        "science/generated/README.md must link the governed generated-science subsurfaces exactly"
    );
}

#[test]
fn policy__contracts__science_generated_docs_policy__generated_current_readme_links_current_snapshot_boundaries_exactly(
) {
    let expected = BTreeSet::from([
        "../README.md".to_string(),
        "../../specs/evidence/README.md".to_string(),
        "../indexes/README.md".to_string(),
        "evidence/README.md".to_string(),
    ]);
    let documented = markdown_link_targets("science/generated/current/README.md");
    assert_eq!(
        expected, documented,
        "science/generated/current/README.md must link the governed current-snapshot boundaries exactly"
    );
}
