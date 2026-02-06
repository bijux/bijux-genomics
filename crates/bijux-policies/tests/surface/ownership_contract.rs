use std::path::{Path, PathBuf};

use walkdir::WalkDir;

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn is_core_ids_path(path: &Path) -> bool {
    path.to_string_lossy()
        .ends_with("/crates/bijux-core/src/ids.rs")
}

fn is_core_metrics_path(path: &Path) -> bool {
    path.to_string_lossy()
        .ends_with("/crates/bijux-core/src/metrics.rs")
}

fn is_pipelines_defaults_path(path: &Path) -> bool {
    path.to_string_lossy()
        .ends_with("/crates/bijux-pipelines/src/defaults_ledger.rs")
}

fn is_domain_params_path(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    path_str.contains("/crates/bijux-domain-") && path_str.contains("/params/")
}

fn is_policies_ownership_test(path: &Path) -> bool {
    path.to_string_lossy()
        .ends_with("/crates/bijux-policies/tests/ownership_contract.rs")
}

#[test]
fn ownership_contract_is_complete() {
    let root = workspace_root();
    let contract_path = root
        .join("crates")
        .join("bijux-core")
        .join("src")
        .join("boundaries.md");
    let content = std::fs::read_to_string(&contract_path).expect("read boundaries.md");
    let required = [
        "## OWNERSHIP",
        "IDs (PipelineId/StageId/ToolId/MetricId):",
        "Defaults/profiles:",
        "Param schemas:",
        "Metric semantics:",
        "Artifact layout:",
        "Report schema/rendering:",
    ];
    let missing: Vec<&str> = required
        .iter()
        .copied()
        .filter(|needle| !content.contains(needle))
        .collect();
    assert!(
        missing.is_empty(),
        "ownership contract missing entries: {:?}",
        missing
    );
}

#[test]
fn id_definitions_live_in_core_only() {
    let root = workspace_root();
    let mut offenders = Vec::new();
    let patterns = [
        "struct StageId",
        "enum StageId",
        "struct ToolId",
        "enum ToolId",
        "struct PipelineId",
        "enum PipelineId",
        "struct MetricId",
        "enum MetricId",
    ];

    for entry in WalkDir::new(root.join("crates"))
        .into_iter()
        .filter_map(|entry| entry.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        if is_core_ids_path(entry.path())
            || is_core_metrics_path(entry.path())
            || is_policies_ownership_test(entry.path())
        {
            continue;
        }
        let content = std::fs::read_to_string(entry.path()).expect("read source");
        for pattern in patterns {
            if content.contains(pattern) {
                offenders.push(format!("{} defines {pattern}", entry.path().display()));
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "ID types must be defined only in bijux-core:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn id_parsing_and_validation_live_in_core_only() {
    let root = workspace_root();
    let mut offenders = Vec::new();
    let regex = regex::Regex::new(r"fn\s+(parse|validate)_[a-z0-9_]*(?:_id|_id_str)\b").unwrap();

    for entry in WalkDir::new(root.join("crates"))
        .into_iter()
        .filter_map(|entry| entry.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        if entry.path().to_string_lossy().contains("/tests/") {
            continue;
        }
        if is_core_ids_path(entry.path())
            || is_core_metrics_path(entry.path())
            || is_policies_ownership_test(entry.path())
        {
            continue;
        }
        let content = std::fs::read_to_string(entry.path()).expect("read source");
        if regex.is_match(&content) {
            offenders.push(entry.path().display().to_string());
        }
    }

    assert!(
        offenders.is_empty(),
        "ID parsing/validation must live in bijux-core:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn defaults_ledger_is_owned_by_pipelines() {
    let root = workspace_root();
    let mut offenders = Vec::new();
    let regex = regex::Regex::new(r"struct\s+DefaultsLedger\w*").unwrap();

    for entry in WalkDir::new(root.join("crates"))
        .into_iter()
        .filter_map(|entry| entry.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        if is_pipelines_defaults_path(entry.path()) {
            continue;
        }
        let content = std::fs::read_to_string(entry.path()).expect("read source");
        if regex.is_match(&content) {
            offenders.push(entry.path().display().to_string());
        }
    }

    assert!(
        offenders.is_empty(),
        "Defaults ledger structs must live in bijux-pipelines:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn no_hidden_param_defaults_outside_domain() {
    let root = workspace_root();
    let mut offenders = Vec::new();
    let regex = regex::Regex::new(r"impl\s+Default\s+for\s+[A-Za-z0-9_]*Params").unwrap();

    for entry in WalkDir::new(root.join("crates"))
        .into_iter()
        .filter_map(|entry| entry.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        if is_domain_params_path(entry.path()) {
            continue;
        }
        let content = std::fs::read_to_string(entry.path()).expect("read source");
        if regex.is_match(&content) {
            offenders.push(entry.path().display().to_string());
        }
    }

    assert!(
        offenders.is_empty(),
        "Default impls for *Params must live in domain crates only:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn registry_of_truth_markers_are_unique() {
    let root = workspace_root();
    let mut matches = Vec::new();
    let patterns = ["registry-of-truth", "registry_of_truth", "RegistryOfTruth"];

    for entry in WalkDir::new(root.join("crates"))
        .into_iter()
        .filter_map(|entry| entry.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        if is_policies_ownership_test(entry.path()) {
            continue;
        }
        let content = std::fs::read_to_string(entry.path()).expect("read source");
        for pattern in patterns {
            if content.contains(pattern) {
                matches.push(format!("{} contains {pattern}", entry.path().display()));
            }
        }
    }

    assert!(
        matches.len() <= 1,
        "registry-of-truth markers should not be duplicated:\n{}",
        matches.join("\n")
    );
}

#[test]
fn id_module_files_live_in_core_only() {
    let root = workspace_root();
    let mut offenders = Vec::new();

    for entry in WalkDir::new(root.join("crates"))
        .into_iter()
        .filter_map(|entry| entry.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        if entry.path().to_string_lossy().contains("/tests/") {
            continue;
        }
        let path_str = entry.path().to_string_lossy();
        if !path_str.ends_with("_id.rs") {
            continue;
        }
        if path_str.contains("/crates/bijux-core/") {
            continue;
        }
        offenders.push(path_str.to_string());
    }

    assert!(
        offenders.is_empty(),
        "*_id.rs files must live in bijux-core only:\n{}",
        offenders.join("\n")
    );
}
