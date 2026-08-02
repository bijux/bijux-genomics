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
        "profile.ci must terminate tests that exceed the one-second CI contract"
    );
    let fast_unit_profile = config
        .split("[profile.fast-unit]\n")
        .nth(1)
        .and_then(|tail| tail.split("\n[profile.").next())
        .expect("profile.fast-unit section");
    bijux_dna_policies::policy_assert!(
        fast_unit_profile
            .contains("slow-timeout = { period = \"1s\", terminate-after = 15 }"),
        "profile.fast-unit must report tests as slow after one second and terminate after 15 seconds"
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
    let shared_cargo_mk =
        std::fs::read_to_string(root.join(".bijux/shared/bijux-makes-rs/cargo.mk"))
            .expect("read shared Rust Make contract");
    let root_mk = std::fs::read_to_string(root.join("makes/root.mk")).expect("read makes/root.mk");
    let rust_gate =
        std::fs::read_to_string(root.join(".bijux/shared/bijux-makes-rs/scripts/rust_gate.sh"))
            .expect("read shared Rust gate");
    let pinned_gate =
        std::fs::read_to_string(root.join(".bijux/shared/bijux-makes/scripts/run_pinned_gate.sh"))
            .expect("read shared pinned gate");
    let nextest_expression_builder = root.join("makes/bin/nextest_expr.sh");
    let nextest_expression_builder_source = std::fs::read_to_string(&nextest_expression_builder)
        .expect("read Nextest expression builder");
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
        full_profile.contains("test-threads = 8"),
        "profile.full must run the complete suite with eight nextest workers"
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
        "test-all must default to the deterministic full nextest profile"
    );
    bijux_dna_policies::policy_assert!(
        cargo_mk.contains("RS_TARGET_DIR ?= $(abspath $(ARTIFACT_ROOT)/target)"),
        "repository and shared Rust gates must reuse one Cargo target directory"
    );
    bijux_dna_policies::policy_assert!(
        cargo_mk.contains("RS_CARGO_HOME ?= $(abspath $(ARTIFACT_ROOT)/cargo/home)"),
        "repository and shared Rust gates must reuse one Cargo dependency cache"
    );
    bijux_dna_policies::policy_assert!(
        cargo_mk.contains("NEXTEST_EXPR_BIN ?= $(CURDIR)/makes/bin/nextest_expr.sh"),
        "make test lanes must use the repository Nextest expression boundary"
    );
    bijux_dna_policies::policy_assert!(
        nextest_expression_builder_source
            .contains(".bijux/shared/bijux-makes-rs/scripts/nextest_expr.sh")
            && nextest_expression_builder_source
                .contains("sed 's#test(/\\^(?:#test(/(?:^|::)(?:#g'"),
        "the repository Nextest expression boundary must preserve the shared builder and match rostered unit-test suffixes"
    );
    bijux_dna_policies::policy_assert!(
        cargo_mk.contains("NEXTEST_SLOW_NAME_EXPR ?= test(/::slow__/)"),
        "make test lanes must use the shared slow__ namespace"
    );
    bijux_dna_policies::policy_assert!(
        root_mk.contains("bijux-makes/environment.mk")
            && root_mk.contains("bijux-makes-rs/bijux.mk"),
        "root Make entrypoint must load the shared common and Rust contracts"
    );
    bijux_dna_policies::policy_assert!(
        shared_cargo_mk.contains("NEXTEST_THREADS_ALL ?= 8"),
        "shared test-all must default complete suites to eight nextest workers"
    );
    bijux_dna_policies::policy_assert!(
        !cargo_mk.contains("NEXTEST_THREADS_ALL"),
        "repository Make policy must not override shared complete-suite concurrency"
    );
    bijux_dna_policies::policy_assert!(
        rust_gate.contains("\"${NEXTEST_THREADS_ALL:-8}\""),
        "shared Rust gate must preserve eight-worker execution when invoked directly"
    );
    bijux_dna_policies::policy_assert!(
        rust_gate.contains("args+=(--run-ignored all --retries 0)"),
        "shared test-all lane must include ignored tests and disable retries"
    );
    bijux_dna_policies::policy_assert!(
        rust_gate.contains("\"nextest-summary:\"") && rust_gate.contains("return \"${status}\""),
        "shared test-all lane must preserve the final nextest summary and exit status"
    );
    for needle in [
        "pinned_ref=\"${PINNED_REF:-${TEST_ALL_FROZEN_REF:-HEAD}}\"",
        "export PROJECT_ROOT=\"${pinned_repo_dir}\"",
        "artifact_execution_root=\"${pinned_repo_dir}/artifacts\"",
        "export ARTIFACT_ROOT=\"${artifact_execution_root}\"",
        "artifact publication conflict:",
        "ln -s ",
    ] {
        bijux_dna_policies::policy_assert!(
            pinned_gate.contains(needle),
            "shared pinned gate must preserve `{needle}`"
        );
    }
    bijux_dna_policies::policy_assert!(
        slow_roster.lines().map(str::trim).any(|line| !line.is_empty() && !line.starts_with('#')),
        "nextest slow roster must contain governed slow test names"
    );
}
