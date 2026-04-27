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
fn policy__contracts__root_science_docs_policy__science_index_links_root_docs_exactly() {
    let expected = BTreeSet::from([
        "SCIENTIFIC_DECISIONS.md".to_string(),
        "SCIENTIFIC_DEFAULTS.md".to_string(),
        "PUBLICATION_ASSETS.md".to_string(),
        "TOOL_INDEX.md".to_string(),
        "DOMAIN_COVERAGE.generated.md".to_string(),
        "TOOL_STAGE_CITATIONS.md".to_string(),
        "VALIDITY_LIMITS.md".to_string(),
        "fastq/index.md".to_string(),
        "bam/index.md".to_string(),
        "vcf/index.md".to_string(),
    ]);
    let documented = markdown_link_targets("docs/20-science/index.md");
    assert_eq!(
        expected, documented,
        "The root science index must link the governed root docs and domain landing pages exactly"
    );
}
