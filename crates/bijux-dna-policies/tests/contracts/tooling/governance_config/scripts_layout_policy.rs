#![allow(non_snake_case)]
use std::path::PathBuf;

use walkdir::WalkDir;

fn workspace_root() -> PathBuf {
    bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn policy__contracts__scripts_layout_policy__legacy_scripts_directory_is_removed() {
    let root = workspace_root();
    let legacy_dir = ["scr", "ipts"].concat();
    bijux_dna_policies::policy_assert!(
        !root.join(&legacy_dir).exists(),
        "legacy automation directory must be fully migrated into bijux-dna-dev and removed"
    );
}

#[test]
fn policy__contracts__scripts_layout_policy__repo_does_not_reference_legacy_scripts() {
    let root = workspace_root();
    let legacy = ["scr", "ipts/"].concat();
    let allowlist = [
        ".github/workflows/automerge-pr.yml",
        ".github/workflows/bijux-std.yml",
        ".github/workflows/github-policy.yml",
        ".github/bijux-std-shared.sha256",
        ".github/scripts/check_protected_github_changes.py",
        ".github/scripts/sync_github_standards.py",
        ".github/standards/repo-config.manifest.json",
    ];
    let mut offenders = Vec::new();
    for scope in [
        root.join("Makefile"),
        root.join("makes"),
        root.join("docs"),
        root.join("examples"),
        root.join("configs"),
        root.join("crates"),
        root.join(".github"),
    ] {
        if scope.is_file() {
            let raw = std::fs::read_to_string(&scope).unwrap_or_default();
            let rel =
                scope.strip_prefix(&root).unwrap_or(&scope).to_string_lossy().replace('\\', "/");
            if raw.contains(&legacy) && !allowlist.iter().any(|allowed| rel == *allowed) {
                offenders.push(scope.display().to_string());
            }
            continue;
        }
        if !scope.is_dir() {
            continue;
        }
        for entry in WalkDir::new(&scope).into_iter().filter_map(Result::ok) {
            if !entry.file_type().is_file() {
                continue;
            }
            let rel = entry
                .path()
                .strip_prefix(&root)
                .unwrap_or(entry.path())
                .to_string_lossy()
                .replace('\\', "/");
            if rel
                == "crates/bijux-dna-policies/tests/contracts/tooling/governance_config/scripts_layout_policy.rs"
            {
                continue;
            }
            if allowlist.iter().any(|allowed| rel == *allowed) {
                continue;
            }
            let raw = std::fs::read_to_string(entry.path()).unwrap_or_default();
            if raw.contains(&legacy) {
                offenders.push(entry.path().display().to_string());
            }
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "legacy automation references remain in repo content:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__scripts_layout_policy__ci_does_not_call_lab_workflows() {
    let root = workspace_root();
    let mut offenders = Vec::new();
    for entry in WalkDir::new(root.join(".github/workflows")).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|s| s.to_str()) != Some("yml") {
            continue;
        }
        let raw = std::fs::read_to_string(entry.path()).unwrap_or_default();
        if raw.contains("cargo run -p bijux-dna-dev -- lab ")
            || raw.contains("cargo run -q -p bijux-dna-dev -- lab ")
        {
            offenders.push(entry.path().display().to_string());
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "CI workflows must not invoke lab workflows directly: {}",
        offenders.join(", ")
    );
}
