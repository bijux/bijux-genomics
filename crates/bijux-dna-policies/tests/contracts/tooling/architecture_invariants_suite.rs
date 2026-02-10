#![allow(non_snake_case)]
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("resolve repo root")
        .to_path_buf()
}

#[test]
fn policy__contracts__architecture_invariants_suite__core_boundary_artifacts_exist() {
    let root = repo_root();
    let required = [
        "docs/10-architecture/ARCHITECTURE.md",
        "docs/10-architecture/CRATE_AUTHORITY_MAP.md",
        "docs/10-architecture/BOUNDARY_MAP.md",
        "crates/bijux-dna-policies/tests/boundaries/deps/dependency_graph.rs",
        "crates/bijux-dna-policies/tests/contracts/tooling/purity_effects_responsibility_policy.rs",
        "crates/bijux-dna-policies/tests/contracts/tooling/generated_configs_policy.rs",
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
