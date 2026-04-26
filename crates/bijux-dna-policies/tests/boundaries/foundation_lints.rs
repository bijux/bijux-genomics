#![allow(non_snake_case)]

use std::path::{Path, PathBuf};

const FOUNDATION_CRATES: &[&str] = &[
    "bijux-dna",
    "bijux-dna-api",
    "bijux-dna-core",
    "bijux-dna-dev",
    "bijux-dna-engine",
    "bijux-dna-infra",
    "bijux-dna-policies",
    "bijux-dna-runner",
    "bijux-dna-runtime",
    "bijux-dna-testkit",
];

#[test]
fn policy__boundaries__foundation_lints__foundation_crates_apply_workspace_lints() {
    let workspace = workspace_root();

    for crate_name in FOUNDATION_CRATES {
        let manifest_path = workspace.join("crates").join(crate_name).join("Cargo.toml");
        let manifest = std::fs::read_to_string(&manifest_path)
            .unwrap_or_else(|err| panic!("read {}: {err}", manifest_path.display()));

        assert!(
            has_workspace_lints(&manifest),
            "{crate_name} must opt into the workspace lint table"
        );
    }
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap_or_else(|| panic!("resolve workspace root"))
        .to_path_buf()
}

fn has_workspace_lints(manifest: &str) -> bool {
    let mut in_lints = false;

    for line in manifest.lines() {
        let line = line.trim();
        if line.starts_with('[') && line.ends_with(']') {
            in_lints = line == "[lints]";
            continue;
        }
        if in_lints && line == "workspace = true" {
            return true;
        }
    }

    false
}
