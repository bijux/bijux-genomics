#![allow(non_snake_case)]
use std::env;
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

#[test]
fn policy__contracts__ci_tools_policy__workflows_use_make_only() {
    let root = workspace_root();
    let workflows_dir = root.join(".github").join("workflows");
    let mut offenders = Vec::new();
    for entry in WalkDir::new(workflows_dir)
        .into_iter()
        .filter_map(|entry| entry.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|s| s.to_str()) != Some("yml") {
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
    let allowed = ["bijux-dna-infra", "bijux-dna-infra"];
    let mut offenders = Vec::new();
    for entry in WalkDir::new(root.join("crates"))
        .into_iter()
        .filter_map(|entry| entry.ok())
    {
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
    let cargo_mk = root.join("makefiles").join("cargo.mk");
    let content = std::fs::read_to_string(&cargo_mk).expect("read cargo.mk");

    let required = [
        "cargo nextest run",
        "--config-file configs/nextest/nextest.toml",
        "--workspace",
        "--all-features",
        "--profile $(NEXTEST_PROFILE)",
        "--run-ignored all",
        "cargo llvm-cov nextest",
        "--no-report",
        "--no-cfg-coverage",
        "cargo llvm-cov report",
        "--json",
        "--html",
    ];

    let mut missing = Vec::new();
    for needle in required {
        if !content.contains(needle) {
            missing.push(needle);
        }
    }

    bijux_dna_policies::policy_assert!(
        missing.is_empty(),
        "CI coverage/test commands must include stable nextest/coverage flags. Missing: {missing:?}"
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

    bijux_dna_policies::policy_assert!(
        !missing,
        "TEST_TARGET_DIR/COV_TARGET_DIR/TEST_TMP_DIR/COV_TMP_DIR/TEST_PROFRAW_DIR/COV_PROFRAW_DIR must be set for CI isolation checks."
    );

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
    let scan_roots = [
        root.join("docs"),
        root.join("scripts"),
        root.join(".github"),
    ];
    let mut offenders = Vec::new();

    for scan_root in scan_roots {
        if !scan_root.exists() {
            continue;
        }
        for entry in WalkDir::new(scan_root)
            .into_iter()
            .filter_map(|entry| entry.ok())
        {
            if !entry.file_type().is_file() {
                continue;
            }
            let content = match std::fs::read_to_string(entry.path()) {
                Ok(content) => content,
                Err(_) => continue,
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
        "Docs/scripts/CI must not reference legacy bijux:: namespace: {:?}",
        offenders
    );
}

#[test]
fn policy__contracts__ci_tools_policy__ci_environment_contract_is_stable() {
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
