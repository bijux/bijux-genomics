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
fn policy__contracts__science_boundary_docs_policy__science_root_readme_links_contract_surface_exactly(
) {
    let expected = BTreeSet::from(["CONTRACT.md".to_string()]);
    let documented = markdown_link_targets("science/README.md")
        .into_iter()
        .filter(|target| target == "CONTRACT.md")
        .collect::<BTreeSet<_>>();
    assert_eq!(
        expected, documented,
        "science/README.md must link the root science contract surface exactly"
    );
}

#[test]
fn policy__contracts__science_boundary_docs_policy__science_contract_links_boundary_surfaces_exactly(
) {
    let expected = BTreeSet::from([
        "README.md".to_string(),
        "specs/data/README.md".to_string(),
        "specs/evidence/README.md".to_string(),
        "specs/results/README.md".to_string(),
        "specs/reports/README.md".to_string(),
        "specs/releases/README.md".to_string(),
        "generated/README.md".to_string(),
        "generated/current/README.md".to_string(),
        "generated/current/evidence/README.md".to_string(),
        "generated/indexes/README.md".to_string(),
        "docs/README.md".to_string(),
        "../domain/fastq/execution_support.yaml".to_string(),
        "../domain/fastq/docs/DEFAULT_SETTINGS.md".to_string(),
        "../configs/ci/registry/tool_registry.toml".to_string(),
        "../crates/bijux-dna-environment/docs/ENV_REFERENCE.md".to_string(),
        "docs/upstream/fastq/tools/EVIDENCE_MAP.tsv".to_string(),
        "docs/upstream/papers/TOOL_PAPER_MAP.tsv".to_string(),
    ]);
    let documented = markdown_link_targets("science/CONTRACT.md");
    assert_eq!(
        expected, documented,
        "science/CONTRACT.md must link the governed boundary surfaces exactly"
    );
}

#[test]
fn policy__contracts__science_boundary_docs_policy__release_manifest_inventory_links_governed_files_exactly(
) {
    let expected = BTreeSet::from([
        "../README.md".to_string(),
        "../CONTRACT.md".to_string(),
        "fastq-environment-baseline.yaml".to_string(),
    ]);
    let documented = markdown_link_targets("science/specs/releases/manifests/README.md");
    assert_eq!(
        expected, documented,
        "science/specs/releases/manifests/README.md must link the governed release-manifest files exactly"
    );
}

#[test]
fn policy__contracts__science_boundary_docs_policy__container_science_boundary_links_governed_surfaces_exactly(
) {
    let expected = BTreeSet::from([
        "../../domain/fastq/execution_support.yaml".to_string(),
        "../../docs/20-science/fastq/REFERENCES.md".to_string(),
        "../../domain/fastq/docs/EVIDENCE_CLOSURE.md".to_string(),
        "../../science/generated/current/evidence/README.md".to_string(),
        "SMOKE_CONTRACT.md".to_string(),
        "PROMOTION_POLICY.md".to_string(),
    ]);
    let documented = markdown_link_targets("containers/docs/SCIENCE_EVIDENCE_BOUNDARY.md");
    assert_eq!(
        expected, documented,
        "containers/docs/SCIENCE_EVIDENCE_BOUNDARY.md must link the governed science and container review surfaces exactly"
    );
}
