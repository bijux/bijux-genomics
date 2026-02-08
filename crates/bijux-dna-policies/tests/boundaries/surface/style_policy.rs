#![allow(non_snake_case)]
#[path = "../../support/fs.rs"]
mod support;

const STYLE_CHECKS: &[&str] = &[
    "docs_required_policy.rs",
    "no_thin_modules_policy.rs",
    "no_helpers_policy.rs",
    "no_src_crowd_policy.rs",
    "mod_naming_policy.rs",
];

#[test]
fn policy__boundaries__style_policy__style_policy_entrypoint_lists_checks() {
    let matrix_path = support::workspace_root().join("docs/40-policies/POLICY_MATRIX.md");
    let matrix = support::read_to_string(&matrix_path);
    let mut missing = Vec::new();
    for check in STYLE_CHECKS {
        if !matrix.contains(check) {
            missing.push(check.to_string());
        }
    }

    bijux_dna_policies::policy_assert!(
        missing.is_empty(),
        "STYLE_POLICY is the entrypoint for style checks, and POLICY_MATRIX.md must list them.\n\
How to fix: add the missing policy filenames to POLICY_MATRIX.md under the Style section.\n\
Missing:\n{}",
        missing.join("\n")
    );
}

#[test]
fn policy__boundaries__style_policy__scope_docs_reference_workspace_style() {
    let mut offenders = Vec::new();
    for crate_root in support::crate_roots() {
        let scope_path = crate_root.join("docs").join("SCOPE.md");
        if !scope_path.exists() {
            continue;
        }
        let content = support::read_to_string(&scope_path);
        if !content.contains("docs/40-policies/STYLE.md") {
            offenders.push(scope_path.display().to_string());
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "SCOPE.md must link to docs/40-policies/STYLE.md:\n{}",
        offenders.join("\n")
    );
}
