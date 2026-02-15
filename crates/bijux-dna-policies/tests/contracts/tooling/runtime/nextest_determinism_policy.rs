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
fn policy__contracts__nextest_determinism_policy__ci_profile_disables_flaky_ordering_behaviors() {
    let root = repo_root();
    let config = std::fs::read_to_string(root.join("configs/nextest/nextest.toml"))
        .expect("read configs/nextest/nextest.toml");
    bijux_dna_policies::policy_assert!(
        config.contains("[profile.ci]"),
        "configs/nextest/nextest.toml must define [profile.ci]"
    );
    bijux_dna_policies::policy_assert!(
        config.contains("test-threads = 1"),
        "profile.ci must enforce test-threads = 1 for ordering independence"
    );
    bijux_dna_policies::policy_assert!(
        config.contains("retries = { count = 0"),
        "profile.ci must disable retries for deterministic failure surfaces"
    );
}
