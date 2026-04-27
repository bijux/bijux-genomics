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

#[test]
fn policy__contracts__science_boundary_docs_policy__container_license_readme_links_governed_surfaces_exactly(
) {
    let expected = BTreeSet::from([
        "../README.md".to_string(),
        "../index.md".to_string(),
        "../docs/VERSION_AUTHORITY.md".to_string(),
        "../docs/SCIENCE_EVIDENCE_BOUNDARY.md".to_string(),
    ]);
    let documented = markdown_link_targets("containers/licenses/README.md");
    assert_eq!(
        expected, documented,
        "containers/licenses/README.md must link the governed container license-review surfaces exactly"
    );
}

#[test]
fn policy__contracts__science_boundary_docs_policy__container_root_readme_links_governed_entrypoints_exactly(
) {
    let expected = BTreeSet::from([
        "index.md".to_string(),
        "docs/index.md".to_string(),
        "docs/TOOL_LIFECYCLE.md".to_string(),
        "docs/VERSION_AUTHORITY.md".to_string(),
        "docs/SCIENCE_EVIDENCE_BOUNDARY.md".to_string(),
        "licenses/README.md".to_string(),
        "docs/GHCR_PUBLISH.md".to_string(),
        "docs/SMOKE_CONTRACT.md".to_string(),
        "docs/PROMOTION_POLICY.md".to_string(),
    ]);
    let documented = markdown_link_targets("containers/README.md");
    assert_eq!(
        expected, documented,
        "containers/README.md must link the governed container entrypoints exactly"
    );
}

#[test]
fn policy__contracts__science_boundary_docs_policy__container_index_links_governed_surfaces_exactly(
) {
    let expected = BTreeSet::from([
        "docs/index.md".to_string(),
        "docs/TOOL_LIFECYCLE.md".to_string(),
        "docs/VERSION_AUTHORITY.md".to_string(),
        "versions/index.md".to_string(),
        "versions/LOCK.md".to_string(),
        "docs/SMOKE_CONTRACT.md".to_string(),
        "docs/PROMOTION_POLICY.md".to_string(),
        "docs/SCIENCE_EVIDENCE_BOUNDARY.md".to_string(),
        "docs/SECURITY_BOUNDARY.md".to_string(),
        "docs/MULTIARCH_POLICY.md".to_string(),
        "licenses/README.md".to_string(),
        "versions/versions.toml".to_string(),
        "versions/lock.json".to_string(),
        "versions/index.sha256".to_string(),
    ]);
    let documented = markdown_link_targets("containers/index.md");
    assert_eq!(
        expected, documented,
        "containers/index.md must link the governed container control surfaces exactly"
    );
}

#[test]
fn policy__contracts__science_boundary_docs_policy__operations_container_doc_links_governed_surfaces_exactly(
) {
    let expected = BTreeSet::from([
        "../50-reference/TOOL_ADMISSION.md".to_string(),
        "../../containers/index.md".to_string(),
        "../../containers/docs/index.md".to_string(),
        "../../containers/README.md".to_string(),
        "../../containers/docs/RELEASE_CHECKLIST.md".to_string(),
        "../../containers/docs/PLANNED.md".to_string(),
    ]);
    let documented = markdown_link_targets("docs/30-operations/CONTAINERS.md");
    assert_eq!(
        expected, documented,
        "docs/30-operations/CONTAINERS.md must link the governed container operations surfaces exactly"
    );
}

#[test]
fn policy__contracts__science_boundary_docs_policy__container_versions_index_links_governed_surfaces_exactly(
) {
    let expected = BTreeSet::from([
        "../README.md".to_string(),
        "../docs/VERSION_AUTHORITY.md".to_string(),
        "../docs/LOCK_LIFECYCLE.md".to_string(),
        "versions.toml".to_string(),
        "LOCK.md".to_string(),
        "lock.json".to_string(),
        "index.sha256".to_string(),
    ]);
    let documented = markdown_link_targets("containers/versions/index.md");
    assert_eq!(
        expected, documented,
        "containers/versions/index.md must link the governed version-control surfaces exactly"
    );
}

#[test]
fn policy__contracts__science_boundary_docs_policy__container_version_authority_links_governed_surfaces_exactly(
) {
    let expected = BTreeSet::from([
        "../README.md".to_string(),
        "../versions/index.md".to_string(),
        "../versions/versions.toml".to_string(),
        "../versions/lock.json".to_string(),
        "../versions/LOCK.md".to_string(),
    ]);
    let documented = markdown_link_targets("containers/docs/VERSION_AUTHORITY.md");
    assert_eq!(
        expected, documented,
        "containers/docs/VERSION_AUTHORITY.md must link the governed version-authority surfaces exactly"
    );
}

#[test]
fn policy__contracts__science_boundary_docs_policy__container_lock_lifecycle_links_governed_surfaces_exactly(
) {
    let expected = BTreeSet::from([
        "../versions/lock.json".to_string(),
        "../README.md".to_string(),
        "VERSION_AUTHORITY.md".to_string(),
        "../versions/LOCK.md".to_string(),
        "../versions/versions.toml".to_string(),
    ]);
    let documented = markdown_link_targets("containers/docs/LOCK_LIFECYCLE.md");
    assert_eq!(
        expected, documented,
        "containers/docs/LOCK_LIFECYCLE.md must link the governed lock-lifecycle surfaces exactly"
    );
}

#[test]
fn policy__contracts__science_boundary_docs_policy__container_version_lock_doc_links_governed_surfaces_exactly(
) {
    let expected = BTreeSet::from([
        "versions.toml".to_string(),
        "lock.json".to_string(),
        "../README.md".to_string(),
        "index.md".to_string(),
        "../docs/VERSION_AUTHORITY.md".to_string(),
        "../docs/FRONTEND_BUILD_AUTHORITY.md".to_string(),
        "deprecations.toml".to_string(),
    ]);
    let documented = markdown_link_targets("containers/versions/LOCK.md");
    assert_eq!(
        expected, documented,
        "containers/versions/LOCK.md must link the governed lock-authority surfaces exactly"
    );
}

#[test]
fn policy__contracts__science_boundary_docs_policy__container_promotion_policy_links_governed_surfaces_exactly(
) {
    let expected = BTreeSet::from([
        "../README.md".to_string(),
        "../index.md".to_string(),
        "VERSION_AUTHORITY.md".to_string(),
        "../apptainer/shared/NON_BIJUX_SOURCES.md".to_string(),
        "../versions/versions.toml".to_string(),
        "../versions/LOCK.md".to_string(),
        "../OWNERS.toml".to_string(),
    ]);
    let documented = markdown_link_targets("containers/docs/PROMOTION_POLICY.md");
    assert_eq!(
        expected, documented,
        "containers/docs/PROMOTION_POLICY.md must link the governed promotion-policy surfaces exactly"
    );
}

#[test]
fn policy__contracts__science_boundary_docs_policy__container_release_checklist_links_governed_surfaces_exactly(
) {
    let expected = BTreeSet::from([
        "../README.md".to_string(),
        "../index.md".to_string(),
        "VERSION_AUTHORITY.md".to_string(),
        "GHCR_PUBLISH.md".to_string(),
        "../../configs/ci/registry/".to_string(),
        "../versions/versions.toml".to_string(),
        "../versions/LOCK.md".to_string(),
        "index.md".to_string(),
    ]);
    let documented = markdown_link_targets("containers/docs/RELEASE_CHECKLIST.md");
    assert_eq!(
        expected, documented,
        "containers/docs/RELEASE_CHECKLIST.md must link the governed release-checklist surfaces exactly"
    );
}

#[test]
fn policy__contracts__science_boundary_docs_policy__container_frontend_build_authority_links_governed_surfaces_exactly(
) {
    let expected = BTreeSet::from([
        "../README.md".to_string(),
        "../versions/LOCK.md".to_string(),
        "../../docs/30-operations/TRACEABILITY_PROOF_FRONTEND.md".to_string(),
        "../versions/versions.toml".to_string(),
        "../../configs/ci/tools/apptainer_cache_policy.toml".to_string(),
    ]);
    let documented = markdown_link_targets("containers/docs/FRONTEND_BUILD_AUTHORITY.md");
    assert_eq!(
        expected, documented,
        "containers/docs/FRONTEND_BUILD_AUTHORITY.md must link the governed frontend-build-authority surfaces exactly"
    );
}

#[test]
fn policy__contracts__science_boundary_docs_policy__frontend_traceability_doc_links_governed_surfaces_exactly(
) {
    let expected = BTreeSet::from([
        "../../containers/docs/FRONTEND_BUILD_AUTHORITY.md".to_string(),
        "../../containers/versions/LOCK.md".to_string(),
        "../../containers/versions/lock.json".to_string(),
        "../../configs/vcf/panels/panels.toml".to_string(),
        "../../configs/vcf/panels/locks/lock.json".to_string(),
    ]);
    let documented = markdown_link_targets("docs/30-operations/TRACEABILITY_PROOF_FRONTEND.md");
    assert_eq!(
        expected, documented,
        "docs/30-operations/TRACEABILITY_PROOF_FRONTEND.md must link the governed frontend-traceability surfaces exactly"
    );
}

#[test]
fn policy__contracts__science_boundary_docs_policy__container_tool_lifecycle_links_governed_surfaces_exactly(
) {
    let expected = BTreeSet::from([
        "../README.md".to_string(),
        "../index.md".to_string(),
        "PROMOTION_POLICY.md".to_string(),
        "../versions/deprecations.toml".to_string(),
        "../TOOL_IDS.txt".to_string(),
    ]);
    let documented = markdown_link_targets("containers/docs/TOOL_LIFECYCLE.md");
    assert_eq!(
        expected, documented,
        "containers/docs/TOOL_LIFECYCLE.md must link the governed lifecycle surfaces exactly"
    );
}
