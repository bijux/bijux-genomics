#![allow(non_snake_case)]
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

fn workspace_root() -> PathBuf {
    bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"))
}

fn contains_root_automation_reference(raw: &str) -> bool {
    static PATHS: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    if !raw.contains("scripts/") {
        return false;
    }
    // Keep the entire path token: a nested scripts directory and a javascripts
    // asset have distinct owners from the repository's root automation directory.
    let paths = PATHS.get_or_init(|| {
        regex::Regex::new(
            r"(?:\$\([A-Za-z_][A-Za-z0-9_]*\)|\$\{[A-Za-z_][A-Za-z0-9_]*\}|[A-Za-z0-9_.$~/\\-])+",
        )
        .expect("automation path token expression")
    });
    paths.find_iter(raw).any(|candidate| {
        let path = candidate.as_str();
        if !path.contains("scripts/") {
            return false;
        }
        let path = [
            "$(CURDIR)/",
            "$(REPO_ROOT)/",
            "${REPO_ROOT}/",
            "${repo_root}/",
            "${PWD}/",
            "$PWD/",
            "$REPO_ROOT/",
        ]
        .iter()
        .find_map(|prefix| path.strip_prefix(prefix))
        .unwrap_or(path);
        let mut components = Vec::new();
        for component in path.split('/') {
            match component {
                "" | "." => {}
                ".." => {
                    if components.pop().is_none() {
                        return false;
                    }
                }
                part => components.push(part),
            }
        }
        components.len() > 1 && components[0] == "scripts"
    })
}

// Test fixtures describe refused paths; production references remain governed.
fn contains_root_automation_reference_in_file(path: &Path, raw: &str) -> bool {
    if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
        let production = bijux_dna_testkit::workspace_support::strip_rust_test_modules(raw);
        contains_root_automation_reference(&production)
    } else {
        contains_root_automation_reference(raw)
    }
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
            if contains_root_automation_reference_in_file(&scope, &raw)
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
            if contains_root_automation_reference_in_file(entry.path(), &raw) {
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
fn policy__contracts__scripts_layout_policy__path_context_preserves_nested_owners() {
    for reference in [
        "assets/javascripts/navigation-sync.js",
        "assets/vendor/mathjax/scripts/render.js",
        "$(DOCS_PUBLICATION_SHARED)/tooling/scripts/docs_source_authority.sh",
        ".bijux/shared/bijux-checks/scripts/validate-shared-contracts.sh",
        "https://example.org/scripts/render.js",
        "~/scripts/local-tool.sh",
    ] {
        assert!(!contains_root_automation_reference(reference), "nested owner: {reference}");
    }
    for reference in [
        "python3 './scripts/check.py'",
        "bash ${REPO_ROOT}/scripts/check.sh",
        "bash $(CURDIR)/scripts/check.sh",
        "bash $REPO_ROOT/scripts/check.sh",
        "[old automation](scripts/check.sh)",
        ".github/../scripts/check.py",
        "makes/../scripts/check.sh",
    ] {
        assert!(contains_root_automation_reference(reference), "root owner: {reference}");
    }
}

#[test]
fn policy__contracts__scripts_layout_policy__rust_test_fixtures_preserve_production_checks() {
    let source = r##"pub fn production() {}
#[cfg(test)]
mod tests {
    fn negative_fixture() {
        let quoted = "} scripts/legacy.sh";
        let raw = r#"} scripts/legacy.sh"#;
        /* } /* { */ */
        assert_eq!(quoted, raw);
    }
}
"##;
    let rust = Path::new("owner.rs");
    assert!(contains_root_automation_reference(source));
    assert!(!contains_root_automation_reference_in_file(rust, source));
    assert!(contains_root_automation_reference_in_file(Path::new("owner.mk"), source));
    assert!(!contains_root_automation_reference_in_file(
        rust,
        "#![cfg(test)]\nconst FIXTURE: &str = \"scripts/legacy.sh\";"
    ));
    for production in [
        format!("const TOOL: &str = \"scripts/legacy.sh\";\n{source}"),
        format!("{source}\nconst TOOL: &str = \"scripts/legacy.sh\";"),
        "const FIXTURE: &str = r#\"\n#[cfg(test)]\nmod fake {}\n\"#;\nconst TOOL: &str = \"scripts/legacy.sh\";".to_owned(),
    ] {
        assert!(contains_root_automation_reference_in_file(rust, &production));
    }
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
