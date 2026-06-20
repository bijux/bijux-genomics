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
        config.contains("slow-timeout = { period = \"1s\", terminate-after = 1 }"),
        "fast nextest profiles must classify tests over 1 second as slow"
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
    let slow_roster = std::fs::read_to_string(root.join("configs/rust/nextest-slow-roster.txt"))
        .expect("read nextest slow roster");
    bijux_dna_policies::policy_assert!(
        config.contains("[profile.full]"),
        "configs/rust/nextest.toml must define [profile.full] for test-all and coverage"
    );
    let full_profile = config
        .split("[profile.full]\n")
        .nth(1)
        .and_then(|tail| tail.split("\n[profile.").next())
        .expect("profile.full section");
    bijux_dna_policies::policy_assert!(
        full_profile.contains("retries = { count = 0, backoff = \"fixed\", delay = \"1s\" }"),
        "profile.full must disable retries for deterministic full-suite surfaces"
    );
    bijux_dna_policies::policy_assert!(
        full_profile.contains("fail-fast = false"),
        "profile.full must keep the complete suite available after failures"
    );
    bijux_dna_policies::policy_assert!(
        full_profile.contains("test-threads = 1"),
        "profile.full must keep the full suite deterministic"
    );
    bijux_dna_policies::policy_assert!(
        full_profile.contains("slow-timeout = \"1s\""),
        "profile.full must classify tests over 1 second as slow during the complete suite"
    );
    bijux_dna_policies::policy_assert!(
        !full_profile.contains("terminate-after"),
        "profile.full must not terminate long-running tests during the complete suite"
    );
    bijux_dna_policies::policy_assert!(
        cargo_mk.contains("NEXTEST_PROFILE_ALL ?= full"),
        "test-all must default to the full nextest profile"
    );
    bijux_dna_policies::policy_assert!(
        cargo_mk.contains("NEXTEST_EXPR_BIN ?= makes/bin/nextest_expr.sh"),
        "make test lanes must derive slow-test filters from the governed expression builder"
    );
    bijux_dna_policies::policy_assert!(
        slow_roster.lines().map(str::trim).any(|line| !line.is_empty() && !line.starts_with('#')),
        "nextest slow roster must contain governed slow test names"
    );
}
