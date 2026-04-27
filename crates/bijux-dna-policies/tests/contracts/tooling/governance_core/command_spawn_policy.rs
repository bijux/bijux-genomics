#![allow(non_snake_case)]
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

fn workspace_root() -> PathBuf {
    bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"))
}

fn is_allowed_command_path(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    path_str.contains("/crates/bijux-dna-runner/")
        || path_str.contains("/crates/bijux-dna-environment/src/build/")
        || path_str.contains("/crates/bijux-dna-environment/src/bin/")
        || path_str.contains("/crates/bijux-dna-environment/src/resolve/")
        || path_str.contains("/crates/bijux-dna-environment-qa/src/bin/")
        || path_str.contains("/crates/bijux-dna-environment-qa/src/image_qa/")
        || path_str.contains("/crates/bijux-dna-dev/")
        || path_str.contains("/crates/bijux-dna-stages-vcf/")
}

#[test]
fn policy__contracts__command_spawn_policy__command_spawning_is_confined_to_runner_and_env_tooling()
{
    let root = workspace_root();
    let mut offenders = Vec::new();
    let needles = ["std::process::Command", "Command::new"];

    for entry in WalkDir::new(root.join("crates")).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }
        if path.to_string_lossy().contains("/tests/") {
            continue;
        }
        if is_allowed_command_path(path) {
            continue;
        }
        let content = std::fs::read_to_string(path).expect("read source");
        if needles.iter().any(|needle| content.contains(needle)) {
            offenders.push(path.display().to_string());
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "std::process::Command must be confined to bijux-dna-runner or bijux-dna-environment tooling:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__command_spawn_policy__crate_tests_do_not_spawn_external_commands() {
    let root = workspace_root();
    let mut offenders = Vec::new();
    let needles = [
        "std::process::Command",
        "Command::new",
        "tokio::process::Command",
        "assert_cmd",
        "duct::cmd",
        "DockerRunner",
        "docker::",
        "bollard::",
        "apptainer",
    ];

    for entry in WalkDir::new(root.join("crates")).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }
        let path_str = path.to_string_lossy();
        if !path_str.contains("/tests/") {
            continue;
        }
        if path_str.contains("/tests/boundaries/") {
            continue;
        }
        if path_str.contains("/crates/bijux-dna-policies/tests/")
            || path_str.contains("/crates/bijux-dna-stages-fastq/tests/architecture.rs")
            || path_str
                .contains("/crates/bijux-dna-api/tests/contracts/v1_fastq_small_integration.rs")
            || path_str.contains("/crates/bijux-dna-runner/tests/boundaries/architecture.rs")
            || path_str
                .contains("/crates/bijux-dna-environment-qa/tests/boundaries/architecture.rs")
            || path_str.contains("/crates/bijux-dna-environment/tests/contracts/resolve_runtime.rs")
        {
            continue;
        }
        let content = std::fs::read_to_string(path).expect("read source");
        if needles.iter().any(|needle| content.contains(needle)) {
            offenders.push(path.display().to_string());
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "crate tests must not spawn external commands (use fixtures or an explicit corpus-root contract instead):\n{}",
        offenders.join("\n")
    );
}
