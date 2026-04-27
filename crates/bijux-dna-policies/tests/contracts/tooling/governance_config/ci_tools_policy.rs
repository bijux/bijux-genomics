#![allow(non_snake_case)]
use std::env;
use std::path::PathBuf;

use walkdir::WalkDir;

fn workspace_root() -> PathBuf {
    bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn policy__contracts__ci_tools_policy__workflows_use_make_only() {
    let root = workspace_root();
    let workflows_dir = root.join(".github").join("workflows");
    let allowlist = [
        ".github/workflows/automerge-pr.yml",
        ".github/workflows/codecov.yml",
        ".github/workflows/github-policy.yml",
        ".github/workflows/labeler.yml",
    ];
    let mut offenders = Vec::new();
    for entry in WalkDir::new(workflows_dir).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|s| s.to_str()) != Some("yml") {
            continue;
        }
        let rel = entry
            .path()
            .strip_prefix(&root)
            .unwrap_or(entry.path())
            .to_string_lossy()
            .replace('\\', "/");
        if allowlist.iter().any(|allowed| rel == *allowed) {
            continue;
        }
        let content = std::fs::read_to_string(entry.path()).expect("read workflow");
        if content.contains("cargo clippy")
            || content.contains("cargo fmt")
            || content.contains("cargo test")
            || content.contains("cargo nextest")
            || content.contains("cargo make")
        {
            offenders.push(entry.path().display().to_string());
        }
        if !content.contains("make ") {
            offenders.push(entry.path().display().to_string());
        }
    }
    offenders.sort();
    offenders.dedup();
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "CI workflows must use Makefile entrypoints only: {:?}",
        offenders
    );
}

#[test]
fn policy__contracts__ci_tools_policy__serde_yaml_is_scoped() {
    let root = workspace_root();
    let allowed = ["bijux-dna-infra", "bijux-dna-policies"];
    let mut offenders = Vec::new();
    for entry in WalkDir::new(root.join("crates")).into_iter().filter_map(Result::ok) {
        if entry.file_name() != "Cargo.toml" {
            continue;
        }
        let content = std::fs::read_to_string(entry.path()).expect("read Cargo.toml");
        if !content.contains("serde_yaml") && !content.contains("serde-yaml") {
            continue;
        }
        let name = entry
            .path()
            .parent()
            .and_then(|path| path.file_name())
            .and_then(|name| name.to_str())
            .unwrap_or("");
        if !allowed.contains(&name) {
            offenders.push(entry.path().display().to_string());
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "serde_yaml must be scoped to {:?}: {:?}",
        allowed,
        offenders
    );
}

#[test]
fn policy__contracts__ci_tools_policy__coverage_command_policy_is_stable() {
    let root = workspace_root();
    let cargo_mk = root.join("makes").join("cargo.mk");
    let cargo_mk_content = std::fs::read_to_string(&cargo_mk).expect("read cargo.mk");
    let rust_gate = root.join("makes").join("bin").join("rust_gate.sh");
    let rust_gate_content = std::fs::read_to_string(&rust_gate).expect("read rust_gate.sh");

    let cargo_mk_required = [
        "coverage-workspace: ## Run the governed coverage control-plane lane.",
        "_coverage:",
        "NEXTEST_CONFIG=\"$(NEXTEST_CONFIG)\"",
        "TEST_FEATURES=\"$(TEST_FEATURES)\"",
        "NEXTEST_PROFILE=\"$(NEXTEST_PROFILE)\"",
        "NEXTEST_TEST_THREADS=\"$(NEXTEST_TEST_THREADS)\"",
        "RUN_IGNORED=\"$(RUN_IGNORED)\"",
        "COVERAGE_OUT=\"$(COVERAGE_OUT)\"",
        "COVERAGE_BASELINE=\"$(COVERAGE_BASELINE)\"",
        "COVERAGE_THRESHOLDS=\"$(COVERAGE_THRESHOLDS)\"",
        "cargo run -q -p bijux-dna-dev -- tooling run ci-coverage",
        "coverage-rs: ## Run Rust coverage with llvm-cov and emit reports.",
        "\"$(RUST_GATE_BIN)\" coverage",
    ];
    let rust_gate_required = [
        "cargo llvm-cov nextest",
        "--workspace",
        "--all-features",
        "--run-ignored all",
        "--config-file \"${nextest_config_file}\"",
        "--profile \"${nextest_profile_all}\"",
        "cargo llvm-cov report --summary-only",
    ];

    let mut missing = Vec::new();
    for needle in cargo_mk_required {
        if !cargo_mk_content.contains(needle) {
            missing.push(format!("cargo.mk missing `{needle}`"));
        }
    }
    for needle in rust_gate_required {
        if !rust_gate_content.contains(needle) {
            missing.push(format!("rust_gate.sh missing `{needle}`"));
        }
    }
    bijux_dna_policies::policy_assert!(
        missing.is_empty(),
        "coverage command policy drift: {missing:?}"
    );
}

#[test]
fn policy__contracts__ci_tools_policy__test_and_coverage_dirs_are_isolated() {
    let test_target = env::var("TEST_TARGET_DIR").unwrap_or_default();
    let cov_target = env::var("COV_TARGET_DIR").unwrap_or_default();
    let test_tmp = env::var("TEST_TMP_DIR").unwrap_or_default();
    let cov_tmp = env::var("COV_TMP_DIR").unwrap_or_default();
    let test_profraw = env::var("TEST_PROFRAW_DIR").unwrap_or_default();
    let cov_profraw = env::var("COV_PROFRAW_DIR").unwrap_or_default();

    let missing = [
        test_target.is_empty(),
        cov_target.is_empty(),
        test_tmp.is_empty(),
        cov_tmp.is_empty(),
        test_profraw.is_empty(),
        cov_profraw.is_empty(),
    ]
    .iter()
    .any(|missing| *missing);

    if missing {
        return;
    }

    bijux_dna_policies::policy_assert!(
        !test_target.is_empty() && !cov_target.is_empty(),
        "TEST_TARGET_DIR and COV_TARGET_DIR must be set."
    );
    bijux_dna_policies::policy_assert!(
        test_tmp != cov_tmp,
        "TEST_TMP_DIR and COV_TMP_DIR must be distinct."
    );
    bijux_dna_policies::policy_assert!(
        test_profraw != cov_profraw,
        "TEST_PROFRAW_DIR and COV_PROFRAW_DIR must be distinct."
    );
}

#[test]
fn policy__contracts__ci_tools_policy__no_bijux_namespace_in_docs_or_scripts() {
    let root = workspace_root();
    let scan_roots = [root.join("docs"), root.join("scripts"), root.join(".github")];
    let mut offenders = Vec::new();

    for scan_root in scan_roots {
        if !scan_root.exists() {
            continue;
        }
        for entry in WalkDir::new(scan_root).into_iter().filter_map(Result::ok) {
            if !entry.file_type().is_file() {
                continue;
            }
            let Ok(content) = std::fs::read_to_string(entry.path()) else {
                continue;
            };
            if content.contains("bijux::") {
                offenders.push(entry.path().display().to_string());
            }
        }
    }

    offenders.sort();
    offenders.dedup();
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "Docs, automation, and CI must not reference legacy bijux:: namespace: {:?}",
        offenders
    );
}

#[test]
fn policy__contracts__ci_tools_policy__ci_environment_contract_is_stable() {
    if env::var_os("CI").is_none() {
        return;
    }
    let tz = env::var("TZ").unwrap_or_default();
    let lc_all = env::var("LC_ALL").unwrap_or_default();
    bijux_dna_policies::policy_assert!(
        tz == "UTC",
        "CI must set TZ=UTC for deterministic tests. Observed: {tz}"
    );
    bijux_dna_policies::policy_assert!(
        lc_all == "C",
        "CI must set LC_ALL=C for deterministic tests. Observed: {lc_all}"
    );
}
