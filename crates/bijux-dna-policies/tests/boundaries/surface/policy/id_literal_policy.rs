#![allow(non_snake_case)]
#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use walkdir::WalkDir;

fn repo_root() -> PathBuf {
    bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"))
}

fn is_allowed_path(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    if path_str.contains("/crates/bijux-dna-stages-") {
        return true;
    }
    if path_str.contains("/crates/bijux-dna-domain-") {
        return true;
    }
    if path_str.contains("/crates/bijux-dna-pipelines/") {
        return true;
    }
    if path_str.contains("/crates/bijux-dna-analyze/") {
        return true;
    }
    if path_str.contains("/crates/bijux-dna-bench/") {
        return true;
    }
    if path_str.contains("/crates/bijux-dna/") {
        return true;
    }
    if path_str.contains("/crates/bijux-dna-api/") {
        return true;
    }
    if path_str.contains("/crates/bijux-dna-planner-fastq/") {
        return true;
    }
    if path_str.contains("/crates/bijux-dna-planner-bam/") {
        return true;
    }
    if path_str.ends_with("/crates/bijux-dna-stage-contract/src/executor_registry.rs") {
        return true;
    }
    if path_str.ends_with("/crates/bijux-dna-core/src/metrics/registry.rs") {
        return true;
    }
    if path_str.ends_with("/crates/bijux-dna-core/src/ids.rs") {
        return true;
    }
    if path_str.ends_with("/crates/bijux-dna-core/src/id_catalog.rs") {
        return true;
    }
    if path_str.contains("/crates/bijux-dna-core/src/id_catalog/stage/") {
        return true;
    }
    if path_str.ends_with("/crates/bijux-dna-runtime/src/manifests.rs") {
        return true;
    }
    if path_str.ends_with("/crates/bijux-dna-runtime/src/manifests/classification.rs") {
        return true;
    }
    if path_str.ends_with("/crates/bijux-dna-stage-contract/src/stage_plugin.rs")
        || path_str.ends_with("/crates/bijux-dna-stage-contract/src/plan_run/mod.rs")
        || path_str.ends_with(
            "/crates/bijux-dna-dev/src/commands/ops/tooling/operator_workflow_maturity.rs",
        )
        || path_str.ends_with(
            "/crates/bijux-dna-dev/src/commands/ops/tooling/scientific_caveat_propagation.rs",
        )
        || path_str
            .ends_with("/crates/bijux-dna-dev/src/commands/ops/tooling/reference_external_data.rs")
        || path_str.ends_with("/crates/bijux-dna-bench-model/src/contract/records.rs")
    {
        return true;
    }
    if path_str.contains("/tests/") {
        return true;
    }
    false
}

#[test]
fn policy__boundaries__id_literal_policy__raw_id_catalog_are_confined_to_registries() {
    let root = repo_root();
    let mut offenders = Vec::new();
    let patterns = ["\"fastq.", "\"bam.", "\"cross.", "\"core."];
    for entry in WalkDir::new(root.join("crates")).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        if is_allowed_path(entry.path()) {
            continue;
        }
        let content = std::fs::read_to_string(entry.path()).expect("read source");
        for pattern in patterns {
            if content.contains(pattern) {
                offenders
                    .push(format!("{} contains raw id literal {pattern}", entry.path().display()));
            }
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "raw stage ids must be confined to registries/stage crates:\n{}",
        offenders.join("\n")
    );
}
