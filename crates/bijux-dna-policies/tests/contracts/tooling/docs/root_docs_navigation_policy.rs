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

#[test]
fn policy__contracts__root_docs_navigation_policy__doc_promises_links_governed_surfaces_exactly() {
    let expected = BTreeSet::from([
        "index.md".to_string(),
        "../10-architecture/index.md".to_string(),
        "../20-science/index.md".to_string(),
        "../30-operations/index.md".to_string(),
        "../40-policies/index.md".to_string(),
        "../50-reference/index.md".to_string(),
        "../40-policies/POLICY_INDEX.md".to_string(),
        "../10-architecture/SNAPSHOT_GOLDEN_CONTRACT.md".to_string(),
    ]);
    let documented = markdown_link_targets("docs/00-intro/DOC_PROMISES.md");
    assert_eq!(
        expected, documented,
        "docs/00-intro/DOC_PROMISES.md must link the governed promise authorities exactly"
    );
}

#[test]
fn policy__contracts__root_docs_navigation_policy__docs_map_links_governed_surfaces_exactly() {
    let expected = BTreeSet::from([
        "../index.md".to_string(),
        "index.md".to_string(),
        "../10-architecture/index.md".to_string(),
        "../20-science/index.md".to_string(),
        "../30-operations/index.md".to_string(),
        "../40-policies/index.md".to_string(),
        "../50-reference/index.md".to_string(),
        "../cli/index.md".to_string(),
        "../../containers/index.md".to_string(),
        "../decisions/TOOL_BINDING_DECISIONS.md".to_string(),
        "REPO_ROOT_MAP.generated.md".to_string(),
    ]);
    let documented = markdown_link_targets("docs/00-intro/DOCS_MAP.md");
    assert_eq!(
        expected, documented,
        "docs/00-intro/DOCS_MAP.md must link the governed documentation map surfaces exactly"
    );
}

#[test]
fn policy__contracts__root_docs_navigation_policy__architecture_index_links_governed_surfaces_exactly(
) {
    let expected = BTreeSet::from([
        "../index.md".to_string(),
        "ARCHITECTURE_OVERVIEW.md".to_string(),
        "ARCHITECTURE.md".to_string(),
        "BOUNDARY_MAP.md".to_string(),
        "CONTRACT_SPINE.md".to_string(),
        "CONTRACT_AUTHORITY_LADDER.md".to_string(),
        "SSOT.md".to_string(),
    ]);
    let documented = markdown_link_targets("docs/10-architecture/index.md");
    assert_eq!(
        expected, documented,
        "docs/10-architecture/index.md must link the governed architecture entry surfaces exactly"
    );
}

#[test]
fn policy__contracts__root_docs_navigation_policy__architecture_overview_links_governed_surfaces_exactly(
) {
    let expected = BTreeSet::from([
        "../../crates/bijux-dna-policies/tests/boundaries/deps/core/dependency_boundaries.rs"
            .to_string(),
        "../../crates/bijux-dna-policies/tests/contracts/data/contract_handshake.rs"
            .to_string(),
        "BOUNDARY_DIAGRAM.md".to_string(),
        "CONTRACT_SPINE.md".to_string(),
    ]);
    let documented = markdown_link_targets("docs/10-architecture/ARCHITECTURE_OVERVIEW.md");
    assert_eq!(
        expected, documented,
        "docs/10-architecture/ARCHITECTURE_OVERVIEW.md must link the governed overview authorities exactly"
    );
}

#[test]
fn policy__contracts__root_docs_navigation_policy__architecture_doc_links_governed_surfaces_exactly(
) {
    let expected = BTreeSet::from([
        "../40-policies/index.md".to_string(),
        "../../domain/fastq/artifacts.yaml".to_string(),
        "../../domain/bam/artifacts.yaml".to_string(),
        "../../domain/fastq/metrics.yaml".to_string(),
        "../../domain/bam/metrics.yaml".to_string(),
        "../../configs/ci/registry/tool_registry.toml".to_string(),
        "../../configs/ci/stages/stages.toml".to_string(),
        "../../configs/ci/tools/images.toml".to_string(),
        "CRATE_AUTHORITY_MAP.md".to_string(),
    ]);
    let documented = markdown_link_targets("docs/10-architecture/ARCHITECTURE.md");
    assert_eq!(
        expected, documented,
        "docs/10-architecture/ARCHITECTURE.md must link the governed architecture authorities exactly"
    );
}

#[test]
fn policy__contracts__root_docs_navigation_policy__policies_index_links_governed_surfaces_exactly() {
    let expected = BTreeSet::from([
        "../index.md".to_string(),
        "POLICY_INDEX.md".to_string(),
        "POLICY_MATRIX.md".to_string(),
        "POLICIES_EXPLAINED.md".to_string(),
        "DOCS_STYLE.md".to_string(),
        "STYLE.md".to_string(),
    ]);
    let documented = markdown_link_targets("docs/40-policies/index.md");
    assert_eq!(
        expected, documented,
        "docs/40-policies/index.md must link the governed policy entry surfaces exactly"
    );
}

#[test]
fn policy__contracts__root_docs_navigation_policy__docs_style_links_governed_surfaces_exactly() {
    let expected = BTreeSet::from([
        "../index.md".to_string(),
        "../00-intro/DOCS_MAP.md".to_string(),
        "../DOCS_GRAPH.toml".to_string(),
        "../10-architecture/CONTRACT_AUTHORITY_LADDER.md".to_string(),
    ]);
    let documented = markdown_link_targets("docs/40-policies/DOCS_STYLE.md");
    assert_eq!(
        expected, documented,
        "docs/40-policies/DOCS_STYLE.md must link the governed docs-style authorities exactly"
    );
}

#[test]
fn policy__contracts__root_docs_navigation_policy__operations_index_links_governed_surfaces_exactly(
) {
    let expected = BTreeSet::from([
        "../index.md".to_string(),
        "CI.md".to_string(),
        "BENCHMARK_VARIANCE.md".to_string(),
        "CONTAINERS.md".to_string(),
        "FRONTEND_MINI_STACK_VALIDATION.md".to_string(),
        "PRODUCTION_GUARANTEES.md".to_string(),
        "DEVELOPER_WORKFLOW.md".to_string(),
        "REPRODUCIBILITY.md".to_string(),
        "RUN_ARTIFACTS.md".to_string(),
        "vcf-downstream-triage.md".to_string(),
        "VCF_REFERENCE_CACHE_POLICY.md".to_string(),
        "VCF_DOWNSTREAM_READINESS_CHECKLIST.md".to_string(),
        "SCOPE_CLOSURE_CHECKLIST.generated.md".to_string(),
        "CERTIFICATION_SCOPE.md".to_string(),
    ]);
    let documented = markdown_link_targets("docs/30-operations/index.md");
    assert_eq!(
        expected, documented,
        "docs/30-operations/index.md must link the governed operations entry surfaces exactly"
    );
}

#[test]
fn policy__contracts__root_docs_navigation_policy__reference_index_links_governed_surfaces_exactly() {
    let expected = BTreeSet::from([
        "../index.md".to_string(),
        "REFERENCE_INDEX.md".to_string(),
        "EXAMPLES.md".to_string(),
        "EXAMPLE_FAILURE_TRIAGE.md".to_string(),
        "EXAMPLE_TEMPLATE.md".to_string(),
        "TOOL_ADMISSION.md".to_string(),
        "LICENSING.md".to_string(),
        "PIPELINES.md".to_string(),
        "CRATE_MAP.md".to_string(),
        "PANEL_COMPATIBILITY_MATRIX.md".to_string(),
        "VCF_DOWNSTREAM_COMPATIBILITY_MATRIX.md".to_string(),
    ]);
    let documented = markdown_link_targets("docs/50-reference/index.md");
    assert_eq!(
        expected, documented,
        "docs/50-reference/index.md must link the governed reference entry surfaces exactly"
    );
}

#[test]
fn policy__contracts__root_docs_navigation_policy__reference_contract_index_links_governed_surfaces_exactly(
) {
    let expected = BTreeSet::from([
        "index.md".to_string(),
        "SCHEMAS_INDEX.md".to_string(),
        "COMPATIBILITY_MATRIX.md".to_string(),
        "EXAMPLES.md".to_string(),
        "NO_ORPHANS.md".to_string(),
        "CRATE_MAP.md".to_string(),
        "PIPELINES.md".to_string(),
        "CONTRACT_VERSIONING.md".to_string(),
    ]);
    let documented = markdown_link_targets("docs/50-reference/REFERENCE_INDEX.md");
    assert_eq!(
        expected, documented,
        "docs/50-reference/REFERENCE_INDEX.md must link the governed reference authorities exactly"
    );
}

#[test]
fn policy__contracts__root_docs_navigation_policy__architecture_contract_links_governed_surfaces_exactly(
) {
    let expected = BTreeSet::from([
        "../../Cargo.toml".to_string(),
        "BOUNDARY_MAP.md".to_string(),
        "CRATE_AUTHORITY_MAP.md".to_string(),
        "CONTRACT_SPINE.md".to_string(),
        "CONTRACT_INDEX.md".to_string(),
    ]);
    let documented = markdown_link_targets("docs/10-architecture/ARCHITECTURE_CONTRACT.md");
    assert_eq!(
        expected, documented,
        "docs/10-architecture/ARCHITECTURE_CONTRACT.md must link the governed architecture contract authorities exactly"
    );
}

#[test]
fn policy__contracts__root_docs_navigation_policy__boundary_map_links_governed_surfaces_exactly() {
    let expected = BTreeSet::from([
        "BOUNDARY_DIAGRAM.md".to_string(),
        "DEPENDENCY_RULES.md".to_string(),
        "../../crates/bijux-dna-policies/tests/boundaries/deps/core/dependency_boundaries.rs"
            .to_string(),
        "../../crates/bijux-dna-policies/tests/boundaries/deps/graph/effect_boundary_map.rs"
            .to_string(),
    ]);
    let documented = markdown_link_targets("docs/10-architecture/BOUNDARY_MAP.md");
    assert_eq!(
        expected, documented,
        "docs/10-architecture/BOUNDARY_MAP.md must link the governed boundary authorities exactly"
    );
}

#[test]
fn policy__contracts__root_docs_navigation_policy__contract_authority_ladder_links_governed_surfaces_exactly(
) {
    let expected = BTreeSet::from([
        "../50-reference/TOOL_ADMISSION.md".to_string(),
        "../20-science/TOOL_INDEX.md".to_string(),
        "../../configs/ci/registry/tool_registry.toml".to_string(),
    ]);
    let documented = markdown_link_targets("docs/10-architecture/CONTRACT_AUTHORITY_LADDER.md");
    assert_eq!(
        expected, documented,
        "docs/10-architecture/CONTRACT_AUTHORITY_LADDER.md must link the governed ladder authorities exactly"
    );
}

#[test]
fn policy__contracts__root_docs_navigation_policy__crate_authority_map_links_governed_surfaces_exactly(
) {
    let expected = BTreeSet::from([
        "BOUNDARY_MAP.md".to_string(),
        "../../crates/bijux-dna-policies/docs/POLICY_DIAGNOSTICS.md".to_string(),
        "../../crates/bijux-dna-policies/tests/boundaries/deps/core/dependency_boundaries.rs"
            .to_string(),
        "../../crates/bijux-dna-policies/tests/boundaries/deps/graph/dependency_graph.rs"
            .to_string(),
        "../../crates/bijux-dna-policies/tests/boundaries/deps/graph/effect_boundary_map.rs"
            .to_string(),
        "../../crates/bijux-dna-policies/tests/contracts/tooling/governance_core/command_spawn_policy.rs"
            .to_string(),
        "../../crates/bijux-dna-policies/tests/contracts/tooling/governance/purity_effects_responsibility_policy.rs"
            .to_string(),
    ]);
    let documented = markdown_link_targets("docs/10-architecture/CRATE_AUTHORITY_MAP.md");
    assert_eq!(
        expected, documented,
        "docs/10-architecture/CRATE_AUTHORITY_MAP.md must link the governed crate authority surfaces exactly"
    );
}

#[test]
fn policy__contracts__root_docs_navigation_policy__contract_index_links_governed_surfaces_exactly() {
    let expected = BTreeSet::from([
        "ARCHITECTURE_CONTRACT.md".to_string(),
        "CRATE_BOUNDARY_CONTRACTS.md".to_string(),
        "BOUNDARY_MAP.md".to_string(),
        "CRATE_AUTHORITY_MAP.md".to_string(),
        "CONTRACT_SPINE.md".to_string(),
        "SSOT.md".to_string(),
        "GENERATED_FILES_CONTRACT.md".to_string(),
        "DRY_RUN_EFFECTS_CONTRACT.md".to_string(),
        "SNAPSHOT_GOLDEN_CONTRACT.md".to_string(),
        "../40-policies/TESTS_STYLE.md".to_string(),
        "../../containers/docs/TOOL_IDS_CONTRACT.md".to_string(),
        "CONTRACT_AUTHORITY.md".to_string(),
        "../30-operations/REPORT_CONTRACT.md".to_string(),
        "../../assets/CONTRACT.md".to_string(),
        "../../containers/docs/SMOKE_CONTRACT.md".to_string(),
        "../../containers/docs/SCIENCE_EVIDENCE_BOUNDARY.md".to_string(),
        "../30-operations/benchmark/workspace-contract.md".to_string(),
        "../20-science/SCIENTIFIC_DECISIONS.md".to_string(),
        "../50-reference/LICENSING.md".to_string(),
        "../50-reference/CONTRACT_COMPATIBILITY.md".to_string(),
        "../../.github/release.env".to_string(),
        "../../.github/workflows/publish-ghcr-container-images.yml".to_string(),
        "../../assets/reference/LOCK.md".to_string(),
        "../50-reference/CONTRACT_VERSIONING.md".to_string(),
        "../40-policies/POLICY_OWNERSHIP.md".to_string(),
    ]);
    let documented = markdown_link_targets("docs/10-architecture/CONTRACT_INDEX.md");
    assert_eq!(
        expected, documented,
        "docs/10-architecture/CONTRACT_INDEX.md must link the governed contract index authorities exactly"
    );
}
