#![allow(non_snake_case)]
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

fn workspace_root() -> PathBuf {
    bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"))
}

fn is_core_ids_path(path: &Path) -> bool {
    path.to_string_lossy().contains("/crates/bijux-dna-core/src/ids/")
}

fn is_core_metrics_path(path: &Path) -> bool {
    path.to_string_lossy().contains("/crates/bijux-dna-core/src/metrics/")
}

fn is_core_id_catalog_stage_path(path: &Path) -> bool {
    path.to_string_lossy().contains("/crates/bijux-dna-core/src/id_catalog/stage/")
}

fn is_pipelines_defaults_path(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    path_str.ends_with("/crates/bijux-dna-pipelines/src/defaults_ledger.rs")
        || path_str.ends_with("/crates/bijux-dna-pipelines/src/defaults/ledger.rs")
}

fn is_domain_params_path(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    path_str.contains("/crates/bijux-dna-domain-") && path_str.contains("/params/")
}

fn is_policies_ownership_test(path: &Path) -> bool {
    path.to_string_lossy()
        .ends_with(
            "/crates/bijux-dna-policies/tests/boundaries/surface/structure_layout/ownership_contract.rs",
        )
}

#[test]
fn policy__boundaries__ownership_contract__ownership_contract_is_complete() {
    let root = workspace_root();
    let contract_path = root.join("crates").join("bijux-dna-core").join("docs").join("BOUNDARY.md");
    let content = std::fs::read_to_string(&contract_path).expect("read boundary doc");
    let required = [
        "Public contract types for execution graphs",
        "Canonical JSON rules",
        "Hashing helpers",
        "Typed identifiers and parsing/validation",
        "Canonical identifier catalogs",
        "Shared metric identifiers",
    ];
    let missing: Vec<&str> =
        required.iter().copied().filter(|needle| !content.contains(needle)).collect();
    bijux_dna_policies::policy_assert!(
        missing.is_empty(),
        "ownership contract missing entries: {:?}",
        missing
    );
}

#[test]
fn policy__boundaries__ownership_contract__id_definitions_live_in_core_only() {
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

    for entry in WalkDir::new(root.join("crates")).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        if is_core_ids_path(entry.path())
            || is_core_metrics_path(entry.path())
            || is_core_id_catalog_stage_path(entry.path())
            || is_policies_ownership_test(entry.path())
            || entry
                .path()
                .to_string_lossy()
                .ends_with("/crates/bijux-dna-stages-vcf/src/invariants.rs")
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

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "ID types must be defined only in bijux-dna-core:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__boundaries__ownership_contract__id_parsing_and_validation_live_in_core_only() {
    let root = workspace_root();
    let mut offenders = Vec::new();
    let regex = regex::Regex::new(r"fn\s+(parse|validate)_[a-z0-9_]*(?:_id|_id_str)\b").unwrap();

    for entry in WalkDir::new(root.join("crates")).into_iter().filter_map(Result::ok) {
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
            || is_core_id_catalog_stage_path(entry.path())
            || is_policies_ownership_test(entry.path())
            || entry
                .path()
                .to_string_lossy()
                .ends_with("/crates/bijux-dna-stages-vcf/src/invariants.rs")
        {
            continue;
        }
        let content = std::fs::read_to_string(entry.path()).expect("read source");
        if regex.is_match(&content) {
            offenders.push(entry.path().display().to_string());
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "ID parsing/validation must live in bijux-dna-core:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__boundaries__ownership_contract__defaults_ledger_is_owned_by_pipelines() {
    let root = workspace_root();
    let mut offenders = Vec::new();
    let regex = regex::Regex::new(r"struct\s+DefaultsLedger\w*").unwrap();

    for entry in WalkDir::new(root.join("crates")).into_iter().filter_map(Result::ok) {
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

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "Defaults ledger structs must live in bijux-dna-pipelines:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__boundaries__ownership_contract__no_hidden_param_defaults_outside_domain() {
    let root = workspace_root();
    let mut offenders = Vec::new();
    let regex = regex::Regex::new(r"impl\s+Default\s+for\s+[A-Za-z0-9_]*Params").unwrap();
    let stages_vcf_allowlist = [
        "crates/bijux-dna-stages-vcf/src/pipeline/qc/stage_params.rs",
        "crates/bijux-dna-stages-vcf/src/pipeline_sections/imputation/imputation_types_and_population_params.rs",
        "crates/bijux-dna-stages-vcf/src/pipeline_sections/runtime/call_and_damage_stages.rs",
        "crates/bijux-dna-stages-vcf/src/pipeline_sections/execution/chunking_and_resume.rs",
        "crates/bijux-dna-stages-vcf/src/pipeline_sections/execution/call_filter_and_gl.rs",
    ];

    for entry in WalkDir::new(root.join("crates")).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        let rel = entry.path().strip_prefix(&root).unwrap_or(entry.path());
        let rel_s = rel.to_string_lossy();
        if stages_vcf_allowlist.iter().any(|allowed| rel_s == *allowed) {
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

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "Default impls for *Params must live in domain crates only:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__boundaries__ownership_contract__registry_of_truth_markers_are_unique() {
    let root = workspace_root();
    let mut matches = Vec::new();
    let patterns = ["registry-of-truth", "registry_of_truth", "RegistryOfTruth"];

    for entry in WalkDir::new(root.join("crates")).into_iter().filter_map(Result::ok) {
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

    bijux_dna_policies::policy_assert!(
        matches.len() <= 1,
        "registry-of-truth markers should not be duplicated:\n{}",
        matches.join("\n")
    );
}

#[test]
fn policy__boundaries__ownership_contract__id_module_files_live_in_core_only() {
    let root = workspace_root();
    let mut offenders = Vec::new();

    for entry in WalkDir::new(root.join("crates")).into_iter().filter_map(Result::ok) {
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
        if path_str.contains("/crates/bijux-dna-core/") {
            continue;
        }
        if path_str.ends_with("/crates/bijux-dna-pipelines/src/registry/pipeline_id.rs") {
            continue;
        }
        if path_str.ends_with("/crates/bijux-dna-pipelines/src/fastq/profiles/profile_by_id.rs") {
            continue;
        }
        offenders.push(path_str.to_string());
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "*_id.rs files must live in bijux-dna-core only:\n{}",
        offenders.join("\n")
    );
}
