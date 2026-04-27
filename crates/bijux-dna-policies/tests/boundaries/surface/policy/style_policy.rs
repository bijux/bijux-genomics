#![allow(non_snake_case)]
#[path = "../../../support/fs.rs"]
mod support;

const STYLE_CHECKS: &[&str] = &[
    "docs_required_policy.rs",
    "no_thin_modules_policy.rs",
    "no_helpers_policy.rs",
    "mod_naming_policy.rs",
];

#[test]
fn policy__boundaries__style_policy__style_policy_entrypoint_lists_checks() {
    let matrix_path = support::workspace_root().join("docs/40-policies/POLICY_MATRIX.md");
    let matrix = support::read_to_string(&matrix_path);
    let mut missing = Vec::new();
    for check in STYLE_CHECKS {
        if !matrix.contains(check) {
            missing.push((*check).to_string());
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
fn policy__boundaries__style_policy__crate_docs_have_style_anchor() {
    let index_path = support::workspace_root()
        .join("crates")
        .join("bijux-dna-policies")
        .join("docs")
        .join("INDEX.md");
    let content = support::read_to_string(&index_path);

    bijux_dna_policies::policy_assert!(
        content.contains("## Core") && content.contains("## Contracts"),
        "bijux-dna-policies docs/INDEX.md must expose the current Core/Contracts docs spine"
    );
}
