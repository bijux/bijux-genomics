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
fn policy__contracts__policy_reference_authority_policy__policy_index_links_governed_surfaces_exactly(
) {
    let expected = BTreeSet::from([
        "POLICY_MATRIX.md".to_string(),
        "../../crates/bijux-dna-policies/tests/".to_string(),
    ]);
    let documented = markdown_link_targets("docs/40-policies/POLICY_INDEX.md");
    assert_eq!(
        expected, documented,
        "docs/40-policies/POLICY_INDEX.md must link the governed policy authorities exactly"
    );
}
