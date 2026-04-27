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
fn policy__contracts__science_authored_specs_policy__science_root_links_authored_surfaces_exactly()
{
    let expected = BTreeSet::from([
        "specs/data/README.md".to_string(),
        "specs/evidence/README.md".to_string(),
        "specs/releases/README.md".to_string(),
        "specs/reports/README.md".to_string(),
        "specs/results/README.md".to_string(),
        "docs/README.md".to_string(),
    ]);
    let documented = markdown_link_targets("science/README.md")
        .into_iter()
        .filter(|target| target.starts_with("specs/") || target == "docs/README.md")
        .collect::<BTreeSet<_>>();
    assert_eq!(
        expected, documented,
        "science/README.md must link the authored science surfaces exactly"
    );
}

#[test]
fn policy__contracts__science_authored_specs_policy__science_root_links_generated_entrypoints_exactly(
) {
    let expected = BTreeSet::from([
        "generated/indexes/science_index.json".to_string(),
        "generated/current/evidence/".to_string(),
    ]);
    let documented = markdown_link_targets("science/README.md")
        .into_iter()
        .filter(|target| target.starts_with("generated/"))
        .collect::<BTreeSet<_>>();
    assert_eq!(
        expected, documented,
        "science/README.md must link the generated science entrypoints exactly"
    );
}

#[test]
fn policy__contracts__science_authored_specs_policy__data_specs_docs_link_contract_and_current_authority_exactly(
) {
    let expected = BTreeSet::from([
        "CONTRACT.md".to_string(),
        "../evidence/README.md".to_string(),
    ]);
    let documented = markdown_link_targets("science/specs/data/README.md");
    assert_eq!(
        expected, documented,
        "science/specs/data/README.md must link the data-spec contract and current authority exactly"
    );
}

#[test]
fn policy__contracts__science_authored_specs_policy__release_specs_docs_link_contract_and_adjacent_surfaces_exactly(
) {
    let expected = BTreeSet::from([
        "CONTRACT.md".to_string(),
        "../reports/README.md".to_string(),
        "../results/README.md".to_string(),
    ]);
    let documented = markdown_link_targets("science/specs/releases/README.md");
    assert_eq!(
        expected, documented,
        "science/specs/releases/README.md must link the release-spec contract and adjacent authored surfaces exactly"
    );
}

#[test]
fn policy__contracts__science_authored_specs_policy__report_specs_docs_link_contract_and_dependencies_exactly(
) {
    let expected = BTreeSet::from([
        "CONTRACT.md".to_string(),
        "../evidence/README.md".to_string(),
        "../results/README.md".to_string(),
    ]);
    let documented = markdown_link_targets("science/specs/reports/README.md");
    assert_eq!(
        expected, documented,
        "science/specs/reports/README.md must link the report-spec contract and its authored dependencies exactly"
    );
}

#[test]
fn policy__contracts__science_authored_specs_policy__result_specs_docs_link_contract_and_current_authority_exactly(
) {
    let expected = BTreeSet::from([
        "CONTRACT.md".to_string(),
        "../evidence/README.md".to_string(),
    ]);
    let documented = markdown_link_targets("science/specs/results/README.md");
    assert_eq!(
        expected, documented,
        "science/specs/results/README.md must link the result-spec contract and current authority exactly"
    );
}

#[test]
fn policy__contracts__science_authored_specs_policy__data_specs_contract_links_boundaries_exactly(
) {
    let expected = BTreeSet::from([
        "README.md".to_string(),
        "../evidence/README.md".to_string(),
        "../../README.md".to_string(),
    ]);
    let documented = markdown_link_targets("science/specs/data/CONTRACT.md");
    assert_eq!(
        expected, documented,
        "science/specs/data/CONTRACT.md must link the data-spec boundaries exactly"
    );
}

#[test]
fn policy__contracts__science_authored_specs_policy__result_specs_contract_links_boundaries_exactly(
) {
    let expected = BTreeSet::from([
        "README.md".to_string(),
        "../evidence/README.md".to_string(),
        "../reports/README.md".to_string(),
        "../../README.md".to_string(),
    ]);
    let documented = markdown_link_targets("science/specs/results/CONTRACT.md");
    assert_eq!(
        expected, documented,
        "science/specs/results/CONTRACT.md must link the result-spec boundaries exactly"
    );
}

#[test]
fn policy__contracts__science_authored_specs_policy__report_specs_contract_links_boundaries_exactly(
) {
    let expected = BTreeSet::from([
        "README.md".to_string(),
        "../evidence/README.md".to_string(),
        "../results/README.md".to_string(),
        "../releases/README.md".to_string(),
        "../../README.md".to_string(),
    ]);
    let documented = markdown_link_targets("science/specs/reports/CONTRACT.md");
    assert_eq!(
        expected, documented,
        "science/specs/reports/CONTRACT.md must link the report-spec boundaries exactly"
    );
}
