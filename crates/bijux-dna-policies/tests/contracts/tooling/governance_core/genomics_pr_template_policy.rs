#![allow(non_snake_case)]
use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
        .to_path_buf()
}

#[test]
fn policy__contracts__genomics_pr_template_policy__default_template_covers_contract_and_evidence_review(
) {
    let root = workspace_root();
    let path = root.join(".github/PULL_REQUEST_TEMPLATE/default.md");
    let raw = std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("read {}", path.display()));

    let mut missing = Vec::new();
    for needle in [
        "## Scope",
        "List key files or surfaces changed.",
        "Call out non-goals and intentionally untouched areas.",
        "## Validation",
        "Fast local checks passed.",
        "Relevant targeted tests passed.",
        "## Contracts and Docs",
        "Contract or schema updates are included where behavior changed.",
        "Generated artifacts are refreshed.",
        "## Release and Risk",
        "Breaking change impact is documented.",
    ] {
        if !raw.contains(needle) {
            missing.push(needle.to_string());
        }
    }

    bijux_dna_policies::policy_assert!(
        missing.is_empty(),
        "default PR template missing contract and evidence review prompts: {:?}",
        missing
    );
}
