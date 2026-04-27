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
fn policy__contracts__science_archive_docs_policy__science_docs_readme_links_archive_contracts_exactly(
) {
    let expected = BTreeSet::from([
        "TODO_DOWNLOAD.md".to_string(),
        "../generated/current/evidence/fastq_download_backlog.tsv".to_string(),
        "upstream/README.md".to_string(),
        "upstream/fastq/README.md".to_string(),
        "upstream/fastq/tools/EVIDENCE_MAP.tsv".to_string(),
        "upstream/papers/README.md".to_string(),
        "upstream/papers/TODO_DOWNLOAD.md".to_string(),
        "upstream/papers/TOOL_PAPER_MAP.tsv".to_string(),
        "upstream/github-repos/README.md".to_string(),
        "upstream/github-repos/MANIFEST.tsv".to_string(),
    ]);
    let documented = markdown_link_targets("science/docs/README.md");
    assert_eq!(
        expected, documented,
        "science/docs/README.md must link the governed archive contracts exactly"
    );
}

#[test]
fn policy__contracts__science_archive_docs_policy__science_upstream_readme_links_subsurfaces_exactly(
) {
    let expected = BTreeSet::from([
        "fastq/README.md".to_string(),
        "fastq/tools/README.md".to_string(),
        "fastq/tools/EVIDENCE_MAP.tsv".to_string(),
        "papers/README.md".to_string(),
        "papers/TODO_DOWNLOAD.md".to_string(),
        "papers/TOOL_PAPER_MAP.tsv".to_string(),
        "github-repos/README.md".to_string(),
        "github-repos/MANIFEST.tsv".to_string(),
    ]);
    let documented = markdown_link_targets("science/docs/upstream/README.md");
    assert_eq!(
        expected, documented,
        "science/docs/upstream/README.md must link the governed upstream archive surfaces exactly"
    );
}

#[test]
fn policy__contracts__science_archive_docs_policy__paper_archive_readme_links_contracts_exactly() {
    let expected = BTreeSet::from([
        "TODO_DOWNLOAD.md".to_string(),
        "TOOL_PAPER_MAP.tsv".to_string(),
    ]);
    let documented = markdown_link_targets("science/docs/upstream/papers/README.md");
    assert_eq!(
        expected, documented,
        "science/docs/upstream/papers/README.md must link the governed paper archive contracts exactly"
    );
}

#[test]
fn policy__contracts__science_archive_docs_policy__github_repo_archive_readme_links_manifest() {
    let expected = BTreeSet::from(["MANIFEST.tsv".to_string()]);
    let documented = markdown_link_targets("science/docs/upstream/github-repos/README.md");
    assert_eq!(
        expected, documented,
        "science/docs/upstream/github-repos/README.md must link the governed repository manifest exactly"
    );
}
