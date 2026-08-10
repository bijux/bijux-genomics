#![allow(non_snake_case)]
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

fn workspace_root() -> PathBuf {
    bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"))
}

fn allowed_test_command_arg(arg: &str) -> bool {
    let normalized = arg.trim();
    normalized.contains("env!(\"CARGO_BIN_EXE_")
        || normalized == "\"cargo\""
        || normalized == "\"bash\""
        || normalized == "\"git\""
}

fn content_spawns_forbidden_test_command(content: &str) -> bool {
    content.lines().any(|line| {
        if !line.contains("Command::new(") {
            return false;
        }
        let arg = line.split("Command::new(").nth(1).and_then(|rest| rest.split(')').next());
        arg.is_none_or(|arg| !allowed_test_command_arg(arg))
    })
}

fn is_allowed_command_path(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    path_str.contains("/crates/bijux-dna-runner/")
        || path_str.contains("/crates/bijux-dna-containers/src/build/")
        || path_str.contains("/crates/bijux-dna-containers/src/bin/")
        || path_str.contains("/crates/bijux-dna-containers/src/resolve/")
        || path_str.contains("/crates/bijux-dna-containers-qa/src/bin/")
        || path_str.contains("/crates/bijux-dna-containers-qa/src/image_qa/")
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
        "std::process::Command must be confined to bijux-dna-runner or bijux-dna-containers tooling:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__command_spawn_policy__crate_tests_do_not_spawn_external_commands() {
    let root = workspace_root();
    let mut offenders = Vec::new();
    let forbidden_runtime_needles = [
        "tokio::process::Command",
        "assert_cmd",
        "duct::cmd",
        "DockerRunner",
        "docker::",
        "bollard::",
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
            || path_str.contains("/crates/bijux-dna-containers-qa/tests/boundaries/architecture.rs")
            || path_str.contains("/crates/bijux-dna-containers/tests/contracts/resolve_runtime.rs")
            || path_str.contains("/crates/bijux-dna-runtime/tests/contracts/run_layout_contract.rs")
            || path_str.contains(
                "/crates/bijux-dna-runtime/tests/schemas/schema/runtime_schema_snapshots.rs",
            )
            || path_str.contains("/crates/bijux-dna-domain-compiler/tests/domain_registry_cli.rs")
            || path_str.contains("/crates/bijux-dna/tests/contracts/cli_library_parity.rs")
        {
            continue;
        }
        let content = std::fs::read_to_string(path).expect("read source");
        if content_spawns_forbidden_test_command(&content)
            || forbidden_runtime_needles.iter().any(|needle| content.contains(needle))
        {
            offenders.push(path.display().to_string());
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "crate tests must not spawn arbitrary external commands; only governed workspace harness commands are allowed:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__command_spawn_policy__bijux_dna_tests_use_prebuilt_cli() {
    let tests_root = workspace_root().join("crates/bijux-dna/tests");
    let mut offenders = Vec::new();

    for entry in WalkDir::new(tests_root).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file()
            || entry.path().extension().and_then(|ext| ext.to_str()) != Some("rs")
        {
            continue;
        }
        let path = entry.path();
        let content = std::fs::read_to_string(path).expect("read integration test");
        if content.contains("Command::new(\"cargo\")") {
            offenders.push(path.display().to_string());
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "bijux-dna integration tests must invoke env!(\"CARGO_BIN_EXE_bijux-dna\") instead of spawning nested Cargo builds:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__command_spawn_policy__run_controls_never_poll_for_progress() {
    let root = workspace_root();
    let operations = std::fs::read_to_string(
        root.join("crates/bijux-dna-api/src/runtime/run/reporting/operations.rs"),
    )
    .expect("read run control operations");
    let failure_injection = std::fs::read_to_string(
        root.join("crates/bijux-dna-api/src/runtime/run/reporting/failure_injection.rs"),
    )
    .expect("read failure injection controls");

    for forbidden in
        ["thread::sleep", "while started.elapsed()", ".join(\"run_control.json\").exists()"]
    {
        bijux_dna_policies::policy_assert!(
            !operations.contains(forbidden) && !failure_injection.contains(forbidden),
            "run controls must persist one observation and return; forbidden polling token: {forbidden}"
        );
    }
}
