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
fn policy__contracts__root_docs_navigation_policy__intro_index_links_governed_surfaces_exactly() {
    let expected = BTreeSet::from([
        "../index.md".to_string(),
        "REPO_ROOT_MAP.generated.md".to_string(),
        "WHAT_IS_BIJUX.md".to_string(),
        "SCOPE.md".to_string(),
        "QUICKSTART.md".to_string(),
        "GLOSSARY.md".to_string(),
        "DOC_PROMISES.md".to_string(),
        "REFUSALS.md".to_string(),
        "DOCS_MAP.md".to_string(),
    ]);
    let documented = markdown_link_targets("docs/00-intro/index.md");
    assert_eq!(
        expected, documented,
        "docs/00-intro/index.md must link the governed intro entry surfaces exactly"
    );
}
