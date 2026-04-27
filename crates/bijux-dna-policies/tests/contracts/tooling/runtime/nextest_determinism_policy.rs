#![allow(non_snake_case)]
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn policy__contracts__nextest_determinism_policy__ci_profile_disables_flaky_ordering_behaviors() {
    let root = repo_root();
    let config = std::fs::read_to_string(root.join("configs/rust/nextest.toml"))
        .expect("read configs/rust/nextest.toml");
    bijux_dna_policies::policy_assert!(
        config.contains("[profile.ci]"),
        "configs/rust/nextest.toml must define [profile.ci]"
    );
    bijux_dna_policies::policy_assert!(
        config.contains("test-threads = 1"),
        "profile.ci must enforce test-threads = 1 for ordering independence"
    );
    bijux_dna_policies::policy_assert!(
        config.contains("retries = { count = 0"),
        "profile.ci must disable retries for deterministic failure surfaces"
    );
    bijux_dna_policies::policy_assert!(
        config.contains("slow-timeout = { period = \"10s\", terminate-after = 1 }"),
        "fast nextest profiles must enforce the 10-second budget contract"
    );
}

#[test]
fn policy__contracts__nextest_determinism_policy__full_profile_keeps_long_running_suite_available()
{
    let root = repo_root();
    let config = std::fs::read_to_string(root.join("configs/rust/nextest.toml"))
        .expect("read configs/rust/nextest.toml");
    let cargo_mk =
        std::fs::read_to_string(root.join("makes/cargo.mk")).expect("read makes/cargo.mk");
    bijux_dna_policies::policy_assert!(
        config.contains("[profile.full]"),
        "configs/rust/nextest.toml must define [profile.full] for test-all and coverage"
    );
    bijux_dna_policies::policy_assert!(
        cargo_mk.contains("NEXTEST_PROFILE_ALL ?= full"),
        "test-all must default to the full nextest profile"
    );
}
