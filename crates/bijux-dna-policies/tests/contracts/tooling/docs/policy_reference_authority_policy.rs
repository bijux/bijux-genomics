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

#[test]
fn policy__contracts__policy_reference_authority_policy__style_links_governed_surfaces_exactly() {
    let expected = BTreeSet::from([
        "DOCS_STYLE.md".to_string(),
        "../../crates/bijux-dna-policies/tests/contracts/tooling/docs/boundary_docs_policy.rs"
            .to_string(),
    ]);
    let documented = markdown_link_targets("docs/40-policies/STYLE.md");
    assert_eq!(
        expected, documented,
        "docs/40-policies/STYLE.md must link the governed style authorities exactly"
    );
}

#[test]
fn policy__contracts__policy_reference_authority_policy__example_template_links_governed_surfaces_exactly(
) {
    let expected = BTreeSet::from([
        "../../examples/_template/example.toml".to_string(),
        "../../examples/POLICY.md".to_string(),
        "../../examples/RECIPE_ONLY.txt".to_string(),
        "EXAMPLE_RUNNER_CONTRACT.md".to_string(),
    ]);
    let documented = markdown_link_targets("docs/50-reference/EXAMPLE_TEMPLATE.md");
    assert_eq!(
        expected, documented,
        "docs/50-reference/EXAMPLE_TEMPLATE.md must link the governed example-template authorities exactly"
    );
}

#[test]
fn policy__contracts__policy_reference_authority_policy__example_runner_contract_links_governed_surfaces_exactly(
) {
    let expected = BTreeSet::from([
        "../../examples/index.yaml".to_string(),
        "../30-operations/RUN_ARTIFACTS.md".to_string(),
        "../30-operations/REPORT_CONTRACT.md".to_string(),
    ]);
    let documented = markdown_link_targets("docs/50-reference/EXAMPLE_RUNNER_CONTRACT.md");
    assert_eq!(
        expected, documented,
        "docs/50-reference/EXAMPLE_RUNNER_CONTRACT.md must link the governed example-runner authorities exactly"
    );
}

#[test]
fn policy__contracts__policy_reference_authority_policy__example_failure_triage_links_governed_surfaces_exactly(
) {
    let expected = BTreeSet::from([
        "EXAMPLE_RUNNER_CONTRACT.md".to_string(),
        "../../examples/POLICY.md".to_string(),
        "../../crates/bijux-dna-dev/docs/COMMANDS.md".to_string(),
        "../30-operations/TEST_FAILURE_TRIAGE.md".to_string(),
    ]);
    let documented = markdown_link_targets("docs/50-reference/EXAMPLE_FAILURE_TRIAGE.md");
    assert_eq!(
        expected, documented,
        "docs/50-reference/EXAMPLE_FAILURE_TRIAGE.md must link the governed example-triage authorities exactly"
    );
}

#[test]
fn policy__contracts__policy_reference_authority_policy__bijux_analyze_contract_links_governed_surfaces_exactly(
) {
    let expected = BTreeSet::from([
        "../30-operations/REPORT_CONTRACT.md".to_string(),
        "../30-operations/EXPLAINABILITY.md".to_string(),
    ]);
    let documented = markdown_link_targets("docs/50-reference/BIJUX_ANALYZE_CONTRACT.md");
    assert_eq!(
        expected, documented,
        "docs/50-reference/BIJUX_ANALYZE_CONTRACT.md must link the governed analyze authorities exactly"
    );
}

#[test]
fn policy__contracts__policy_reference_authority_policy__bijux_contract_links_governed_surfaces_exactly(
) {
    let expected = BTreeSet::from([
        "../30-operations/RUN_ARTIFACTS.md".to_string(),
        "../30-operations/REPRODUCIBILITY.md".to_string(),
    ]);
    let documented = markdown_link_targets("docs/50-reference/BIJUX_CONTRACT.md");
    assert_eq!(
        expected, documented,
        "docs/50-reference/BIJUX_CONTRACT.md must link the governed platform authorities exactly"
    );
}

#[test]
fn policy__contracts__policy_reference_authority_policy__pipelines_links_governed_surfaces_exactly(
) {
    let expected = BTreeSet::from([
        "../20-science/SCIENTIFIC_DEFAULTS.md".to_string(),
        "../../crates/bijux-dna-core/src/id_catalog/pipeline/".to_string(),
        "COMPATIBILITY_MATRIX.md".to_string(),
    ]);
    let documented = markdown_link_targets("docs/50-reference/PIPELINES.md");
    assert_eq!(
        expected, documented,
        "docs/50-reference/PIPELINES.md must link the governed pipeline authorities exactly"
    );
}

#[test]
fn policy__contracts__policy_reference_authority_policy__contract_versioning_links_governed_surfaces_exactly(
) {
    let expected = BTreeSet::from([
        "SCHEMAS_INDEX.md".to_string(),
        "COMPATIBILITY_MATRIX.md".to_string(),
    ]);
    let documented = markdown_link_targets("docs/50-reference/CONTRACT_VERSIONING.md");
    assert_eq!(
        expected, documented,
        "docs/50-reference/CONTRACT_VERSIONING.md must link the governed contract-versioning authorities exactly"
    );
}

#[test]
fn policy__contracts__policy_reference_authority_policy__contract_compatibility_links_governed_surfaces_exactly(
) {
    let expected = BTreeSet::from([
        "CONTRACT_VERSIONING.md".to_string(),
        "COMPATIBILITY_MATRIX.md".to_string(),
    ]);
    let documented = markdown_link_targets("docs/50-reference/CONTRACT_COMPATIBILITY.md");
    assert_eq!(
        expected, documented,
        "docs/50-reference/CONTRACT_COMPATIBILITY.md must link the governed contract-compatibility authorities exactly"
    );
}

#[test]
fn policy__contracts__policy_reference_authority_policy__design_authority_links_governed_surfaces_exactly(
) {
    let expected = BTreeSet::from([
        "../10-architecture/BOUNDARY_MAP.md".to_string(),
        "../10-architecture/CONTRACT_AUTHORITY_LADDER.md".to_string(),
        "../40-policies/POLICY_INDEX.md".to_string(),
    ]);
    let documented = markdown_link_targets("docs/50-reference/DESIGN_AUTHORITY.md");
    assert_eq!(
        expected, documented,
        "docs/50-reference/DESIGN_AUTHORITY.md must link the governed design authorities exactly"
    );
}

#[test]
fn policy__contracts__policy_reference_authority_policy__policies_explained_links_governed_surfaces_exactly(
) {
    let expected = BTreeSet::from([
        "POLICY_INDEX.md".to_string(),
        "POLICY_MATRIX.md".to_string(),
        "FAILURE_PLAYBOOKS.md".to_string(),
    ]);
    let documented = markdown_link_targets("docs/40-policies/POLICIES_EXPLAINED.md");
    assert_eq!(
        expected, documented,
        "docs/40-policies/POLICIES_EXPLAINED.md must link the governed policy-explainer authorities exactly"
    );
}

#[test]
fn policy__contracts__policy_reference_authority_policy__policy_ownership_links_governed_surfaces_exactly(
) {
    let expected = BTreeSet::from([
        "../../crates/bijux-dna-policies/README.md".to_string(),
        "POLICY_INDEX.md".to_string(),
        "../10-architecture/CONTRACT_INDEX.md".to_string(),
    ]);
    let documented = markdown_link_targets("docs/40-policies/POLICY_OWNERSHIP.md");
    assert_eq!(
        expected, documented,
        "docs/40-policies/POLICY_OWNERSHIP.md must link the governed policy-ownership authorities exactly"
    );
}

#[test]
fn policy__contracts__policy_reference_authority_policy__policy_stability_links_governed_surfaces_exactly(
) {
    let expected = BTreeSet::from([
        "POLICY_INDEX.md".to_string(),
        "../50-reference/CONTRACT_VERSIONING.md".to_string(),
    ]);
    let documented = markdown_link_targets("docs/40-policies/POLICY_STABILITY.md");
    assert_eq!(
        expected, documented,
        "docs/40-policies/POLICY_STABILITY.md must link the governed policy-stability authorities exactly"
    );
}

#[test]
fn policy__contracts__policy_reference_authority_policy__policy_matrix_links_governed_surfaces_exactly(
) {
    let expected = BTreeSet::from([
        "POLICY_INDEX.md".to_string(),
        "../../crates/bijux-dna-policies/tests/".to_string(),
    ]);
    let documented = markdown_link_targets("docs/40-policies/POLICY_MATRIX.md");
    assert_eq!(
        expected, documented,
        "docs/40-policies/POLICY_MATRIX.md must link the governed policy-matrix authorities exactly"
    );
}

#[test]
fn policy__contracts__policy_reference_authority_policy__failure_playbooks_links_governed_surfaces_exactly(
) {
    let expected = BTreeSet::from([
        "POLICY_INDEX.md".to_string(),
        "POLICY_MATRIX.md".to_string(),
        "../../crates/bijux-dna-policies/docs/ENFORCEMENT.md".to_string(),
    ]);
    let documented = markdown_link_targets("docs/40-policies/FAILURE_PLAYBOOKS.md");
    assert_eq!(
        expected, documented,
        "docs/40-policies/FAILURE_PLAYBOOKS.md must link the governed failure-playbook authorities exactly"
    );
}
