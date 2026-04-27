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
fn policy__contracts__operations_reference_authority_policy__benchmark_variance_links_governed_surfaces_exactly(
) {
    let expected = BTreeSet::from(["../../configs/bench/knobs.toml".to_string()]);
    let documented = markdown_link_targets("docs/30-operations/BENCHMARK_VARIANCE.md");
    assert_eq!(
        expected, documented,
        "docs/30-operations/BENCHMARK_VARIANCE.md must link the governed variance authority exactly"
    );
}

#[test]
fn policy__contracts__operations_reference_authority_policy__hpc_lunarc_layout_links_governed_surfaces_exactly(
) {
    let expected = BTreeSet::from([
        "../../configs/bench/benchmark.toml".to_string(),
        "benchmark/workspace-contract.md".to_string(),
        "benchmark/workspace-model.md".to_string(),
        "RUN_ARTIFACTS.md".to_string(),
    ]);
    let documented = markdown_link_targets("docs/30-operations/HPC_LUNARC_LAYOUT.md");
    assert_eq!(
        expected, documented,
        "docs/30-operations/HPC_LUNARC_LAYOUT.md must link the governed Lunarc workspace authorities exactly"
    );
}

#[test]
fn policy__contracts__operations_reference_authority_policy__reproducibility_links_governed_surfaces_exactly(
) {
    let expected = BTreeSet::from([
        "RUN_ARTIFACTS.md".to_string(),
        "../../configs/vcf/panels/panels.toml".to_string(),
        "../../configs/vcf/panels/locks/lock.json".to_string(),
        "../../configs/vcf/panels/locks/lock.json.sha256".to_string(),
        "../../crates/bijux-dna-bench/bench/suites/".to_string(),
    ]);
    let documented = markdown_link_targets("docs/30-operations/REPRODUCIBILITY.md");
    assert_eq!(
        expected, documented,
        "docs/30-operations/REPRODUCIBILITY.md must link the governed reproducibility authorities exactly"
    );
}

#[test]
fn policy__contracts__operations_reference_authority_policy__vcf_reference_cache_policy_links_governed_surfaces_exactly(
) {
    let expected = BTreeSet::from([
        "../../configs/vcf/panels/panels.toml".to_string(),
        "../../configs/vcf/panels/locks/lock.json".to_string(),
        "../../configs/runtime/profiles/vcf_downstream_local.toml".to_string(),
    ]);
    let documented = markdown_link_targets("docs/30-operations/VCF_REFERENCE_CACHE_POLICY.md");
    assert_eq!(
        expected, documented,
        "docs/30-operations/VCF_REFERENCE_CACHE_POLICY.md must link the governed VCF cache authorities exactly"
    );
}

#[test]
fn policy__contracts__operations_reference_authority_policy__vcf_downstream_readiness_links_governed_surfaces_exactly(
) {
    let expected = BTreeSet::from([
        "VCF_REFERENCE_CACHE_POLICY.md".to_string(),
        "TRACEABILITY_PROOF_FRONTEND.md".to_string(),
        "../../configs/vcf/downstream_acceptance.toml".to_string(),
    ]);
    let documented =
        markdown_link_targets("docs/30-operations/VCF_DOWNSTREAM_READINESS_CHECKLIST.md");
    assert_eq!(
        expected, documented,
        "docs/30-operations/VCF_DOWNSTREAM_READINESS_CHECKLIST.md must link the governed VCF readiness authorities exactly"
    );
}

#[test]
fn policy__contracts__operations_reference_authority_policy__frontend_mini_stack_validation_links_governed_surfaces_exactly(
) {
    let expected = BTreeSet::from([
        "../../examples/index.yaml".to_string(),
        "../50-reference/EXAMPLE_RUNNER_CONTRACT.md".to_string(),
        "EXPLAINABILITY.md".to_string(),
        "REPORT_CONTRACT.md".to_string(),
    ]);
    let documented =
        markdown_link_targets("docs/30-operations/FRONTEND_MINI_STACK_VALIDATION.md");
    assert_eq!(
        expected, documented,
        "docs/30-operations/FRONTEND_MINI_STACK_VALIDATION.md must link the governed frontend-mini authorities exactly"
    );
}

#[test]
fn policy__contracts__operations_reference_authority_policy__production_guarantees_links_governed_surfaces_exactly(
) {
    let expected = BTreeSet::from([
        "CI.md".to_string(),
        "../50-reference/TOOL_ADMISSION.md".to_string(),
        "ISOLATION.md".to_string(),
        "DOCS_BUILD_REPRODUCIBLE.md".to_string(),
        "REPRODUCIBILITY.md".to_string(),
    ]);
    let documented = markdown_link_targets("docs/30-operations/PRODUCTION_GUARANTEES.md");
    assert_eq!(
        expected, documented,
        "docs/30-operations/PRODUCTION_GUARANTEES.md must link the governed production authorities exactly"
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

#[test]
fn policy__contracts__operations_reference_authority_policy__no_orphans_links_governed_surfaces_exactly(
) {
    let expected = BTreeSet::from(["REFERENCE_INDEX.md".to_string()]);
    let documented = markdown_link_targets("docs/50-reference/NO_ORPHANS.md");
    assert_eq!(
        expected, documented,
        "docs/50-reference/NO_ORPHANS.md must link the governed reference index exactly"
    );
}

#[test]
fn policy__contracts__operations_reference_authority_policy__schemas_index_links_governed_surfaces_exactly(
) {
    let expected = BTreeSet::from([
        "../../crates/bijux-dna-api/tests/snapshots".to_string(),
        "../../crates/bijux-dna/tests/snapshots".to_string(),
        "../../crates/bijux-dna-core/tests/schemas".to_string(),
        "../../crates/bijux-dna-stage-contract/tests/schemas".to_string(),
    ]);
    let documented = markdown_link_targets("docs/50-reference/SCHEMAS_INDEX.md");
    assert_eq!(
        expected, documented,
        "docs/50-reference/SCHEMAS_INDEX.md must link the governed schema snapshot surfaces exactly"
    );
}

#[test]
fn policy__contracts__operations_reference_authority_policy__tool_admission_links_governed_surfaces_exactly(
) {
    let expected = BTreeSet::from([
        "../../domain/fastq/tools".to_string(),
        "../../domain/bam/tools".to_string(),
        "../../domain/vcf/tools".to_string(),
        "../../domain/fastq/index.yaml".to_string(),
        "../../domain/bam/index.yaml".to_string(),
        "../../domain/vcf/index.yaml".to_string(),
        "../../configs/ci/registry/tool_registry.toml".to_string(),
        "../../configs/ci/tools/images.toml".to_string(),
        "../../containers/index.md".to_string(),
        "../20-science/TOOL_INDEX.md".to_string(),
        "../30-operations/index.md".to_string(),
        "../../examples/index.yaml".to_string(),
        "../30-operations/CI.md".to_string(),
    ]);
    let documented = markdown_link_targets("docs/50-reference/TOOL_ADMISSION.md");
    assert_eq!(
        expected, documented,
        "docs/50-reference/TOOL_ADMISSION.md must link the governed admission authorities exactly"
    );
}
