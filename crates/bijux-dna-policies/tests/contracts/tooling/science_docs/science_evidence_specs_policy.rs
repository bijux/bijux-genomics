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
fn policy__contracts__science_evidence_specs_policy__evidence_readme_links_governed_inputs_exactly(
) {
    let expected = BTreeSet::from([
        "CONTRACT.md".to_string(),
        "../../docs/upstream/fastq/STAGE_CLAIMS.tsv".to_string(),
        "../../docs/upstream/fastq/STAGE_LIBRARY_SUPPORT.tsv".to_string(),
        "../../generated/README.md".to_string(),
        "../../generated/current/evidence/README.md".to_string(),
        "../../generated/indexes/README.md".to_string(),
    ]);
    let documented = markdown_link_targets("science/specs/evidence/README.md");
    assert_eq!(
        expected, documented,
        "science/specs/evidence/README.md must link the governed authored inputs and generated indexes exactly"
    );
}

#[test]
fn policy__contracts__science_evidence_specs_policy__evidence_contract_links_governed_boundaries_exactly(
) {
    let expected = BTreeSet::from([
        "README.md".to_string(),
        "../../generated/README.md".to_string(),
        "../../generated/current/evidence/README.md".to_string(),
        "../../generated/indexes/README.md".to_string(),
    ]);
    let documented = markdown_link_targets("science/specs/evidence/CONTRACT.md");
    assert_eq!(
        expected, documented,
        "science/specs/evidence/CONTRACT.md must link the governed authored and generated evidence boundaries exactly"
    );
}
