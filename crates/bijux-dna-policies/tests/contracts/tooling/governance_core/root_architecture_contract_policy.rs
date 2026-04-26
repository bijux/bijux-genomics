#![allow(non_snake_case)]

use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap_or_else(|| bijux_dna_policies::policy_panic!("resolve repository root"))
        .to_path_buf()
}

#[test]
fn policy__contracts__root_architecture_contract_policy__root_contract_declares_reviewable_authority()
{
    let root = repo_root();
    let path = root.join("docs/10-architecture/ARCHITECTURE_CONTRACT.md");
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| bijux_dna_policies::policy_panic!("read {}: {err}", path.display()));
    let required = [
        "Owner: Architecture",
        "Scope: Repository root architecture map and boundary authority",
        "Contract version: v1",
        "## Allowed inputs",
        "## Forbidden dependencies",
        "## Forbidden effects",
        "## Validation commands",
        "docs/10-architecture/BOUNDARY_MAP.md",
        "docs/10-architecture/CRATE_AUTHORITY_MAP.md",
        "docs/10-architecture/CONTRACT_SPINE.md",
        "cargo test -p bijux-dna-policies --test boundaries",
        "cargo test -p bijux-dna-policies --test contracts",
    ];

    let missing = required
        .into_iter()
        .filter(|needle| !content.contains(needle))
        .collect::<Vec<_>>();

    bijux_dna_policies::policy_assert!(
        missing.is_empty(),
        "root architecture contract is missing required authority fields:\n{}",
        missing.join("\n")
    );
}
