#![allow(non_snake_case)]
use std::path::PathBuf;

use walkdir::WalkDir;

fn workspace_root() -> PathBuf {
    bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"))
}

fn contains_root_automation_reference(raw: &str) -> bool {
    let automation_segment = ["scr", "ipts/"].concat();
    raw.match_indices(&automation_segment).any(|(index, _)| {
        let prefix = &raw[..index];
        ![".github/", "bijux-makes/", "bijux-makes-rs/"]
            .iter()
            .any(|governed_parent| prefix.ends_with(governed_parent))
    })
}

#[test]
fn policy__contracts__scripts_layout_policy__root_automation_directory_is_absent() {
    let root = workspace_root();
    let root_automation_dir = ["scr", "ipts"].concat();
    bijux_dna_policies::policy_assert!(
        !root.join(&root_automation_dir).exists(),
        "root automation must be owned by bijux-dna-dev or a governed shared library"
    );
}

#[test]
fn policy__contracts__scripts_layout_policy__repo_does_not_reference_root_automation() {
    let root = workspace_root();
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
            if contains_root_automation_reference(&raw)
                && !allowlist.iter().any(|allowed| rel == *allowed)
            {
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
            if contains_root_automation_reference(&raw) {
                offenders.push(entry.path().display().to_string());
            }
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "root automation references remain in repo content:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__scripts_layout_policy__governed_script_libraries_are_not_root_automation() {
    assert!(!contains_root_automation_reference(
        "python3 .github/scripts/check_workflow_prerequisites.py"
    ));
    assert!(!contains_root_automation_reference(
        "$(BIJUX_MAKES_SHARED_ROOT)/bijux-makes-rs/scripts/nextest_expr.sh"
    ));
    assert!(contains_root_automation_reference("python3 scripts/check_workflow_prerequisites.py"));
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
