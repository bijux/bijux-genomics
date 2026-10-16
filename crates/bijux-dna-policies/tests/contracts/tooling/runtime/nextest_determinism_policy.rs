#![allow(non_snake_case)]
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"))
}

fn profile<'a>(config: &'a toml::Value, name: &str) -> &'a toml::Value {
    config.get("profile").and_then(|profiles| profiles.get(name)).expect("named nextest profile")
}

fn retry_count(value: &toml::Value) -> Option<i64> {
    value.as_integer().or_else(|| value.get("count").and_then(toml::Value::as_integer))
}

fn profile_contract_violations(
    config: &toml::Value,
    name: &str,
    complete: bool,
) -> Vec<&'static str> {
    let profile = profile(config, name);
    let mut violations = Vec::new();
    if profile.get("retries").and_then(retry_count) != Some(0) {
        violations.push("profile must explicitly disable retries");
    }
    if complete && profile.get("fail-fast").and_then(toml::Value::as_bool) != Some(false) {
        violations.push("complete profile must collect all failures");
    }
    let timeout = profile.get("slow-timeout");
    let period = timeout.and_then(|value| {
        value.as_str().or_else(|| value.get("period").and_then(toml::Value::as_str))
    });
    if period.is_none_or(|period| period.trim().is_empty()) {
        violations.push("profile must report slow tests");
    }
    let termination = timeout.and_then(|value| value.get("terminate-after"));
    if complete {
        if termination.is_some() {
            violations.push("complete profile must not terminate slow tests");
        }
    } else if termination.and_then(toml::Value::as_integer).is_none_or(|count| count <= 0) {
        violations.push("bounded profile must terminate stalled tests");
    }
    for override_config in
        profile.get("overrides").and_then(toml::Value::as_array).into_iter().flatten().chain(
            config
                .get("profile")
                .and_then(|profiles| profiles.get("default"))
                .filter(|_| name != "default")
                .and_then(|defaults| defaults.get("overrides"))
                .and_then(toml::Value::as_array)
                .into_iter()
                .flatten(),
        )
    {
        if override_config.get("retries").is_some_and(|value| retry_count(value) != Some(0)) {
            violations.push("profile override must not enable retries");
        }
        if complete
            && override_config
                .get("slow-timeout")
                .is_some_and(|value| value.get("terminate-after").is_some())
        {
            violations.push("complete profile override must not terminate slow tests");
        }
    }
    violations
}

#[test]
fn policy__contracts__nextest_determinism_policy__ci_profile_disables_flaky_ordering_behaviors() {
    let root = repo_root();
    let source = std::fs::read_to_string(root.join("configs/rust/nextest.toml"))
        .expect("read configs/rust/nextest.toml");
    let config: toml::Value = toml::from_str(&source).expect("parse nextest profiles");
    for name in ["ci", "fast-unit"] {
        let violations = profile_contract_violations(&config, name, false);
        bijux_dna_policies::policy_assert!(
            violations.is_empty(),
            "profile.{name} violates deterministic bounded execution: {violations:?}"
        );
    }
}

#[test]
fn policy__contracts__nextest_determinism_policy__profile_contracts_bind_effective_settings() {
    let source = r#"
[profile.ci]
retries = 0
test-threads = 4
slow-timeout = { period = "2s", terminate-after = 7 }
[profile.full]
retries = { count = 0, backoff = "fixed", delay = "1s" }
fail-fast = false
test-threads = "num-cpus"
slow-timeout = { period = "3s" }
"#;
    let config: toml::Value = toml::from_str(source).expect("parse profile contract fixture");
    assert!(profile_contract_violations(&config, "ci", false).is_empty());
    assert!(profile_contract_violations(&config, "full", true).is_empty());

    let mut retried_ci = config.clone();
    retried_ci["profile"]["ci"]["retries"] = toml::Value::Integer(1);
    assert!(profile_contract_violations(&retried_ci, "ci", false)
        .contains(&"profile must explicitly disable retries"));

    let mut stopped_full = config.clone();
    stopped_full["profile"]["full"]["fail-fast"] = toml::Value::Boolean(true);
    assert!(profile_contract_violations(&stopped_full, "full", true)
        .contains(&"complete profile must collect all failures"));

    let mut stalled_ci = config.clone();
    stalled_ci["profile"]["ci"]["slow-timeout"]["terminate-after"] = toml::Value::Integer(0);
    assert!(profile_contract_violations(&stalled_ci, "ci", false)
        .contains(&"bounded profile must terminate stalled tests"));

    let overridden: toml::Value = toml::from_str(&format!(
        "{source}\n[[profile.full.overrides]]\nfilter = 'test(example)'\nretries = 1\nslow-timeout = {{ period = '1s', terminate-after = 2 }}\n"
    ))
    .expect("parse profile override fixture");
    let violations = profile_contract_violations(&overridden, "full", true);
    assert!(violations.contains(&"profile override must not enable retries"));
    assert!(violations.contains(&"complete profile override must not terminate slow tests"));

    let inherited: toml::Value = toml::from_str(&format!(
        "{source}\n[[profile.default.overrides]]\nfilter = 'test(example)'\nretries = 1\nslow-timeout = {{ period = '1s', terminate-after = 2 }}\n"
    ))
    .expect("parse inherited override fixture");
    let violations = profile_contract_violations(&inherited, "full", true);
    assert!(violations.contains(&"profile override must not enable retries"));
    assert!(violations.contains(&"complete profile override must not terminate slow tests"));
}

#[test]
fn policy__contracts__nextest_determinism_policy__full_profile_keeps_long_running_suite_available()
{
    let root = repo_root();
    let source = std::fs::read_to_string(root.join("configs/rust/nextest.toml"))
        .expect("read configs/rust/nextest.toml");
    let config: toml::Value = toml::from_str(&source).expect("parse nextest profiles");
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
    let violations = profile_contract_violations(&config, "full", true);
    bijux_dna_policies::policy_assert!(
        violations.is_empty(),
        "profile.full violates complete first-attempt execution: {violations:?}"
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
    // Workflow admission and child lifecycle are exercised by makes/tests/test_github_workflow_gate.py.
    bijux_dna_policies::policy_assert!(
        cargo_mk.contains("NEXTEST_SLOW_NAME_EXPR ?= test(/::slow__/)"),
        "make test lanes must use the shared slow__ namespace"
    );
    assert_shared_full_suite_contract(
        &cargo_mk,
        &shared_cargo_mk,
        &root_mk,
        &rust_gate,
        &pinned_gate,
        &slow_roster,
    );
}

fn assert_shared_full_suite_contract(
    cargo_mk: &str,
    shared_cargo_mk: &str,
    root_mk: &str,
    rust_gate: &str,
    pinned_gate: &str,
    slow_roster: &str,
) {
    bijux_dna_policies::policy_assert!(
        root_mk.contains("bijux-makes/environment.mk")
            && root_mk.contains("bijux-makes-rs/bijux.mk"),
        "root Make entrypoint must load the shared common and Rust contracts"
    );
    bijux_dna_policies::policy_assert!(
        shared_cargo_mk.lines().any(|line| line.trim_start().starts_with("NEXTEST_THREADS_ALL ?=")),
        "shared test-all must expose overridable complete-suite concurrency"
    );
    bijux_dna_policies::policy_assert!(
        !cargo_mk.contains("NEXTEST_THREADS_ALL"),
        "repository Make policy must not override shared complete-suite concurrency"
    );
    bijux_dna_policies::policy_assert!(
        rust_gate.contains("${NEXTEST_THREADS_ALL"),
        "shared Rust gate must consume configured complete-suite concurrency"
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

fn benchmark_ownership_violations(config: &toml::Value) -> Vec<&'static str> {
    let mut violations = Vec::new();
    if config
        .get("test-groups")
        .and_then(|groups| groups.get("benchmark-repository-mutators"))
        .and_then(|group| group.get("max-threads"))
        .and_then(toml::Value::as_integer)
        != Some(1)
    {
        violations.push("shared repository writers must serialize");
    }
    let shared = profile(config, "default")
        .get("overrides")
        .and_then(toml::Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default();
    let mut shared_writers = false;
    let mut private_exceptions = false;
    for override_config in shared {
        match override_config.get("test-group").and_then(toml::Value::as_str) {
            Some("benchmark-repository-mutators") => shared_writers = true,
            Some("@global") => {
                private_exceptions = true;
                if shared_writers {
                    violations
                        .push("qualified private exceptions must precede shared writer rules");
                }
            }
            _ => {}
        }
    }
    if !shared_writers || !private_exceptions {
        violations.push("default profile must own shared writers and qualified private exceptions");
    }
    for (name, value) in
        config.get("profile").and_then(toml::Value::as_table).expect("profile table")
    {
        if name != "default"
            && value
                .get("overrides")
                .and_then(toml::Value::as_array)
                .into_iter()
                .flatten()
                .any(|rule| rule.get("test-group").is_some())
        {
            violations.push("profile-local ownership must not mask the inherited shared boundary");
        }
    }
    violations
}

#[test]
fn policy__contracts__nextest_determinism_policy__benchmark_ownership_is_shared_across_profiles() {
    let root = repo_root();
    let config: toml::Value = toml::from_str(
        &std::fs::read_to_string(root.join("configs/rust/nextest.toml")).expect("read profiles"),
    )
    .expect("parse profiles");
    let violations = benchmark_ownership_violations(&config);
    assert!(violations.is_empty(), "shared benchmark ownership: {violations:?}");
    // Pinned Nextest chains each custom profile's overrides before default overrides.
    // With no local ownership masks, these profiles share the same ordered owner rules.
    for name in ["default", "full", "fast-unit", "slow-integration"] {
        let _ = profile(&config, name);
    }
    let expression = |mode: &str| {
        let output = std::process::Command::new(root.join("makes/bin/nextest_expr.sh"))
            .arg(mode)
            .env("PROJECT_ROOT", &root)
            .env("NEXTEST_SLOW_ROSTER", root.join("configs/rust/nextest-slow-roster.txt"))
            .env("NEXTEST_SLOW_NAME_EXPR", "test(/::slow__/)")
            .output()
            .expect("run existing Nextest expression builder");
        assert!(
            output.status.success(),
            "Nextest expression builder: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).expect("Nextest expression UTF-8").trim().to_string()
    };
    let slow = expression("slow");
    assert_eq!(expression("fast"), format!("not ({slow})"));
    let roster_pattern = slow
        .rsplit_once("test(/")
        .expect("rostered slow predicate")
        .1
        .split_once("/)")
        .expect("closed slow predicate")
        .0;
    let roster = regex::Regex::new(roster_pattern).expect("valid slow roster regex");
    for name in [
        "loaded_hpc_asset_staging_manifest_matches_governed_render",
        "rendered_hpc_asset_staging_manifest_covers_all_domain_jobs",
        "validated_hpc_asset_staging_manifest_rejects_stale_selected_job_count",
    ] {
        assert!(roster.is_match(name), "bare slow case must remain selected: {name}");
        assert!(
            roster.is_match(&format!(
                "commands::benchmark::local_hpc_asset_staging_manifest::tests::{name}"
            )),
            "module-qualified slow case must remain selected: {name}"
        );
        assert!(
            !roster.is_match(&format!("{name}_neighbor")),
            "adjacent unqualified case must not be reclassified"
        );
    }
}

#[test]
fn policy__contracts__nextest_determinism_policy__ownership_controls_refuse_profile_masks() {
    let source = r"
[test-groups.benchmark-repository-mutators]
max-threads = 1
[profile.default]
[[profile.default.overrides]]
filter = 'test(=qualified_private_case)'
test-group = '@global'
[[profile.default.overrides]]
filter = 'package(bijux-dna)'
test-group = 'benchmark-repository-mutators'
[profile.full]
[profile.fast-unit]
[profile.slow-integration]
";
    let config: toml::Value = toml::from_str(source).expect("ownership fixture");
    assert!(benchmark_ownership_violations(&config).is_empty());
    let mut masked = config.clone();
    masked["profile"]["fast-unit"]
        .as_table_mut()
        .expect("fast table")
        .insert("overrides".to_string(), config["profile"]["default"]["overrides"].clone());
    assert!(benchmark_ownership_violations(&masked)
        .contains(&"profile-local ownership must not mask the inherited shared boundary"));
    let mut full_only = config.clone();
    let moved = full_only["profile"]["default"]
        .as_table_mut()
        .expect("default table")
        .remove("overrides")
        .expect("default overrides");
    full_only["profile"]["full"]
        .as_table_mut()
        .expect("full table")
        .insert("overrides".to_string(), moved);
    assert!(benchmark_ownership_violations(&full_only)
        .contains(&"default profile must own shared writers and qualified private exceptions"));
    let mut reordered = config.clone();
    reordered["profile"]["default"]["overrides"].as_array_mut().expect("overrides").reverse();
    assert!(benchmark_ownership_violations(&reordered)
        .contains(&"qualified private exceptions must precede shared writer rules"));
    let mut concurrent = config;
    concurrent["test-groups"]["benchmark-repository-mutators"]["max-threads"] =
        toml::Value::Integer(2);
    assert!(benchmark_ownership_violations(&concurrent)
        .contains(&"shared repository writers must serialize"));
}
