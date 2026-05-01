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
fn policy__contracts__genomics_pr_template_policy__default_template_covers_governed_genomics_surfaces(
) {
    let root = workspace_root();
    let path = root.join(".github/PULL_REQUEST_TEMPLATE/default.md");
    let raw = std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("read {}", path.display()));

    let mut missing = Vec::new();
    for needle in [
        "## Genomics Surfaces",
        "Affected domains:",
        "Affected stages or workflow profiles:",
        "Affected planner, runtime, API, analyze, or evidence surfaces:",
        "Advisory vs enforced scope:",
        "## Fixtures and Evidence",
        "Canonical examples or refusal bundles touched:",
        "Golden, schema, snapshot, or generated registry outputs touched:",
        "Runtime, artifact, evidence, or replay contracts touched:",
        "Domain/stage/planner/runtime/evidence ownership stayed in the correct crate.",
    ] {
        if !raw.contains(needle) {
            missing.push(needle.to_string());
        }
    }

    bijux_dna_policies::policy_assert!(
        missing.is_empty(),
        "default PR template missing governed genomics prompts: {:?}",
        missing
    );
}
