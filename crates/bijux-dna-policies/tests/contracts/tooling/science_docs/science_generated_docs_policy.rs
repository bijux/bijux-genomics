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

fn directory_file_names(path: &str) -> BTreeSet<String> {
    let root = support::workspace_root();
    fs::read_dir(root.join(path))
        .unwrap_or_else(|err| panic!("read {path}: {err}"))
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .filter_map(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .map(str::to_string)
        })
        .collect()
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

#[test]
fn policy__contracts__science_generated_docs_policy__generated_evidence_readme_links_traceability_and_source_ledgers_exactly(
) {
    let expected = BTreeSet::from([
        "../README.md".to_string(),
        "../../indexes/README.md".to_string(),
        "binding_resolution.tsv".to_string(),
        "claim_evidence_map.tsv".to_string(),
        "decision_reasoning_map.tsv".to_string(),
        "fastq_closure_gate.tsv".to_string(),
        "fastq_container_reference_matrix.tsv".to_string(),
        "fastq_default_binding_risk_ledger.tsv".to_string(),
        "fastq_download_backlog.tsv".to_string(),
        "fastq_missing_closure_prerequisites.tsv".to_string(),
        "fastq_paper_archive_matrix.tsv".to_string(),
        "fastq_stage_tool_environment_matrix.tsv".to_string(),
        "fastq_truth_delta.tsv".to_string(),
        "source_inventory.tsv".to_string(),
        "source_archive_gaps.tsv".to_string(),
        "unresolved_refs.json".to_string(),
    ]);
    let documented = markdown_link_targets("science/generated/current/evidence/README.md");
    assert_eq!(
        expected, documented,
        "science/generated/current/evidence/README.md must link the governed traceability and source ledgers exactly"
    );
}

#[test]
fn policy__contracts__science_generated_docs_policy__generated_evidence_readme_lists_all_emitted_ledgers_exactly(
) {
    let expected = directory_file_names("science/generated/current/evidence")
        .into_iter()
        .filter(|name| name != "README.md")
        .collect::<BTreeSet<_>>();
    let documented = markdown_link_targets("science/generated/current/evidence/README.md")
        .into_iter()
        .filter(|target| !target.contains('/'))
        .collect::<BTreeSet<_>>();
    assert_eq!(
        expected, documented,
        "science/generated/current/evidence/README.md must inventory every emitted ledger in science/generated/current/evidence/"
    );
}
