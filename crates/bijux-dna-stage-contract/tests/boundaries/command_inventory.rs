use std::collections::BTreeSet;

const OPERATIONS: &[&str] = &[
    "build-execution-plan",
    "validate-execution-plan",
    "validate-execution-plan-strict",
    "canonical-execution-plan-json",
    "hash-execution-plan",
    "default-stage-edges",
    "stage-plan-json",
    "stage-plan-execution-step",
    "stage-plan-execution-step-with-id",
    "build-run-execution-plan",
    "build-stage-plan",
    "build-tool-execution-spec",
    "validate-stage-outputs",
    "artifact-kind-schema",
    "list-executor-entries",
    "has-stage-executor",
    "stage-executor-entry",
];

#[test]
fn commands_doc_is_complete_stage_contract_inventory() {
    let root = crate::support::crate_root("bijux-dna-stage-contract")
        .unwrap_or_else(|err| panic!("resolve crate root: {err}"));
    let commands_doc = root.join("docs/COMMANDS.md");
    let commands = std::fs::read_to_string(&commands_doc)
        .unwrap_or_else(|err| panic!("read {}: {err}", commands_doc.display()));

    assert!(
        commands.contains("This file is the SSOT for commands and callable operations owned by"),
        "COMMANDS.md must identify itself as the operation SSOT"
    );
    assert!(
        commands.contains("## Managed Operation Inventory"),
        "COMMANDS.md must expose the managed operation inventory"
    );
    assert_eq!(
        documented_operations(&commands),
        OPERATIONS.iter().map(|operation| (*operation).to_string()).collect(),
        "COMMANDS.md must list the exact callable operation inventory"
    );
    for forbidden in [
        "No Cargo binary targets or `src/bin` command modules.",
        "No CLI parser ownership.",
        "No process spawning.",
        "No runtime command execution.",
        "No Docker, Apptainer, or environment command ownership.",
    ] {
        assert!(commands.contains(forbidden), "COMMANDS.md must document `{forbidden}`");
    }
    assert!(
        !root.join("src/bin").exists(),
        "stage-contract must not define Cargo binary command entrypoints"
    );
    assert_local_verification_commands(&commands);
}

fn documented_operations(commands: &str) -> BTreeSet<String> {
    commands
        .lines()
        .filter_map(|line| line.strip_prefix("| `"))
        .filter_map(|line| line.split_once('`').map(|(operation, _)| operation.to_string()))
        .collect()
}

fn assert_local_verification_commands(commands: &str) {
    for command in [
        "CARGO_TARGET_DIR=artifacts/cargo-target cargo check -p bijux-dna-stage-contract --no-default-features",
        "CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-stage-contract --test boundaries --no-default-features",
        "CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-stage-contract --test contracts --no-default-features",
        "CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-stage-contract --test schemas --no-default-features",
    ] {
        assert!(
            commands.contains(command),
            "COMMANDS.md must list focused local verification command `{command}`"
        );
    }
}
