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
fn policy__contracts__operations_reference_authority_policy__security_links_governed_surfaces_exactly(
) {
    let expected = BTreeSet::from(["THREAT_MODEL.md".to_string()]);
    let documented = markdown_link_targets("docs/30-operations/SECURITY.md");
    assert_eq!(
        expected, documented,
        "docs/30-operations/SECURITY.md must link the governed security authority exactly"
    );
}

#[test]
fn policy__contracts__operations_reference_authority_policy__ci_links_governed_surfaces_exactly() {
    let expected = BTreeSet::from([
        "../../configs/rust/nextest.toml".to_string(),
        "../../configs/coverage/runner.toml".to_string(),
        "ISOLATION.md".to_string(),
    ]);
    let documented = markdown_link_targets("docs/30-operations/CI.md");
    assert_eq!(
        expected, documented,
        "docs/30-operations/CI.md must link the governed CI authorities exactly"
    );
}

#[test]
fn policy__contracts__operations_reference_authority_policy__artifact_explorer_links_governed_surfaces_exactly(
) {
    let expected = BTreeSet::from([
        "RUN_ARTIFACTS.md".to_string(),
        "../10-architecture/DATAFLOW.md".to_string(),
        "REPORT_CONTRACT.md".to_string(),
    ]);
    let documented = markdown_link_targets("docs/30-operations/ARTIFACT_EXPLORER.md");
    assert_eq!(
        expected, documented,
        "docs/30-operations/ARTIFACT_EXPLORER.md must link the governed artifact authorities exactly"
    );
}

#[test]
fn policy__contracts__operations_reference_authority_policy__corpus_01_links_governed_surfaces_exactly(
) {
    let expected =
        BTreeSet::from(["../../configs/runtime/corpora/corpus-01.toml".to_string()]);
    let documented = markdown_link_targets("docs/30-operations/corpus-01.md");
    assert_eq!(
        expected, documented,
        "docs/30-operations/corpus-01.md must link the governed corpus specification exactly"
    );
}

#[test]
fn policy__contracts__operations_reference_authority_policy__release_hygiene_links_governed_surfaces_exactly(
) {
    let expected = BTreeSet::from(["../50-reference/CONTRACT_VERSIONING.md".to_string()]);
    let documented = markdown_link_targets("docs/30-operations/RELEASE_HYGIENE.md");
    assert_eq!(
        expected, documented,
        "docs/30-operations/RELEASE_HYGIENE.md must link the governed release versioning authority exactly"
    );
}

#[test]
fn policy__contracts__operations_reference_authority_policy__mkdocs_build_redirect_links_governed_surfaces_exactly(
) {
    let expected = BTreeSet::from(["DOCS_BUILD_REPRODUCIBLE.md".to_string()]);
    let documented = markdown_link_targets("docs/30-operations/MKDOCS_BUILD.md");
    assert_eq!(
        expected, documented,
        "docs/30-operations/MKDOCS_BUILD.md must link the governed docs build authority exactly"
    );
}

#[test]
fn policy__contracts__operations_reference_authority_policy__docs_build_reproducible_links_governed_surfaces_exactly(
) {
    let expected = BTreeSet::from([
        "../../configs/docs/requirements.txt".to_string(),
        "../../configs/docs/mkdocs.toml".to_string(),
        "../../mkdocs.yml".to_string(),
    ]);
    let documented = markdown_link_targets("docs/30-operations/DOCS_BUILD_REPRODUCIBLE.md");
    assert_eq!(
        expected, documented,
        "docs/30-operations/DOCS_BUILD_REPRODUCIBLE.md must link the governed docs build inputs exactly"
    );
}

#[test]
fn policy__contracts__operations_reference_authority_policy__developer_workflow_redirect_links_governed_surfaces_exactly(
) {
    let expected = BTreeSet::from(["../30-operations/DEVELOPER_WORKFLOW.md".to_string()]);
    let documented = markdown_link_targets("docs/40-policies/DEVELOPER_WORKFLOW.md");
    assert_eq!(
        expected, documented,
        "docs/40-policies/DEVELOPER_WORKFLOW.md must link the governed workflow authority exactly"
    );
}
