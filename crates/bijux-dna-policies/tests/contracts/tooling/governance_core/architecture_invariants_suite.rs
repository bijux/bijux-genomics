#![allow(non_snake_case)]
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn policy__contracts__architecture_invariants_suite__core_boundary_artifacts_exist() {
    let root = repo_root();
    let required = [
        "docs/10-architecture/ARCHITECTURE.md",
        "docs/10-architecture/CRATE_AUTHORITY_MAP.md",
        "docs/10-architecture/BOUNDARY_MAP.md",
        "crates/bijux-dna-policies/tests/boundaries/deps/graph/dependency_graph.rs",
        "crates/bijux-dna-policies/tests/contracts/tooling/governance/purity_effects_responsibility_policy.rs",
        "crates/bijux-dna-policies/tests/contracts/tooling/governance_quality/generated_configs_policy.rs",
    ];
    let mut missing = Vec::new();
    for rel in required {
        let path = root.join(rel);
        if !path.exists() {
            missing.push(rel.to_string());
        }
    }
    bijux_dna_policies::policy_assert!(
        missing.is_empty(),
        "architecture invariants suite prerequisites missing:\n{}",
        missing.join("\n")
    );
}
