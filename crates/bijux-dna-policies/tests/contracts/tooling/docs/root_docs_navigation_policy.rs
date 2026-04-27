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

#[test]
fn policy__contracts__root_docs_navigation_policy__what_is_bijux_links_governed_surfaces_exactly() {
    let expected = BTreeSet::from([
        "QUICKSTART.md".to_string(),
        "SCOPE.md".to_string(),
        "../10-architecture/ARCHITECTURE_OVERVIEW.md".to_string(),
        "../50-reference/LICENSING.md".to_string(),
    ]);
    let documented = markdown_link_targets("docs/00-intro/WHAT_IS_BIJUX.md");
    assert_eq!(
        expected, documented,
        "docs/00-intro/WHAT_IS_BIJUX.md must link the governed identity surfaces exactly"
    );
}

#[test]
fn policy__contracts__root_docs_navigation_policy__quickstart_links_governed_surfaces_exactly() {
    let expected = BTreeSet::from([
        "../50-reference/PIPELINES.md".to_string(),
        "../30-operations/RUN_ARTIFACTS.md".to_string(),
    ]);
    let documented = markdown_link_targets("docs/00-intro/QUICKSTART.md");
    assert_eq!(
        expected, documented,
        "docs/00-intro/QUICKSTART.md must link the governed quickstart authorities exactly"
    );
}

#[test]
fn policy__contracts__root_docs_navigation_policy__scope_links_governed_surfaces_exactly() {
    let expected = BTreeSet::from([
        "../10-architecture/index.md".to_string(),
        "../20-science/index.md".to_string(),
        "../30-operations/index.md".to_string(),
        "../50-reference/index.md".to_string(),
    ]);
    let documented = markdown_link_targets("docs/00-intro/SCOPE.md");
    assert_eq!(
        expected, documented,
        "docs/00-intro/SCOPE.md must link the governed scope surfaces exactly"
    );
}

#[test]
fn policy__contracts__root_docs_navigation_policy__glossary_links_governed_surfaces_exactly() {
    let expected = BTreeSet::from([
        "../30-operations/RUN_ARTIFACTS.md".to_string(),
        "../10-architecture/CONTRACT_SPINE.md".to_string(),
        "../30-operations/REPORT_CONTRACT.md".to_string(),
        "../50-reference/PIPELINES.md".to_string(),
        "../40-policies/STYLE.md".to_string(),
    ]);
    let documented = markdown_link_targets("docs/00-intro/GLOSSARY.md");
    assert_eq!(
        expected, documented,
        "docs/00-intro/GLOSSARY.md must link the governed glossary authorities exactly"
    );
}

#[test]
fn policy__contracts__root_docs_navigation_policy__refusals_links_governed_surfaces_exactly() {
    let expected = BTreeSet::from([
        "../10-architecture/BOUNDARY_MAP.md".to_string(),
        "../../domain/fastq/route_policies.toml".to_string(),
        "../20-science/fastq/STAGE_CATALOG.md".to_string(),
    ]);
    let documented = markdown_link_targets("docs/00-intro/REFUSALS.md");
    assert_eq!(
        expected, documented,
        "docs/00-intro/REFUSALS.md must link the governed refusal authorities exactly"
    );
}
