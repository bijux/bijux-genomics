use std::collections::BTreeSet;

const API_COMMANDS: &[&str] = &[
    "plan",
    "execute",
    "execute-and-report",
    "dry-run",
    "status",
    "pause-run",
    "resume-run",
    "cancel-run",
    "operator-health",
    "explain",
    "policy-audit",
    "render-report",
    "render-report-html",
    "workspace-edges",
    "write-workspace-audit",
];

const API_NAMESPACES: &[&str] = &[
    "api::bench",
    "api::plan",
    "api::run",
    "api::report",
    "api::bam",
    "api::fastq",
    "api::env",
    "api::shared",
];

#[test]
fn commands_doc_is_complete_api_inventory() {
    let root = crate::support::crate_root("bijux-dna-api")
        .unwrap_or_else(|err| panic!("resolve crate root: {err}"));
    let commands_doc = root.join("docs/COMMANDS.md");
    let commands = std::fs::read_to_string(&commands_doc)
        .unwrap_or_else(|err| panic!("read {}: {err}", commands_doc.display()));

    assert!(
        commands.contains("This file is the SSOT for commands and callable operations owned by"),
        "COMMANDS.md must identify itself as the API command SSOT"
    );
    assert!(
        commands.contains("## Managed API commands"),
        "COMMANDS.md must expose the managed API command inventory"
    );
    assert_eq!(
        table_commands(&commands),
        API_COMMANDS.iter().map(|command| (*command).to_string()).collect(),
        "docs/COMMANDS.md must list the complete managed API command set"
    );
    assert_eq!(
        documented_namespaces(&commands),
        API_NAMESPACES.iter().map(|namespace| (*namespace).to_string()).collect(),
        "docs/COMMANDS.md must list the curated public v1 helper namespaces"
    );
    assert_local_verification_commands(&commands);
}

fn table_commands(commands: &str) -> BTreeSet<String> {
    commands
        .lines()
        .filter_map(|line| line.strip_prefix("| `"))
        .filter_map(|line| line.split_once('`').map(|(operation, _)| operation.to_string()))
        .collect()
}

fn documented_namespaces(commands: &str) -> BTreeSet<String> {
    commands
        .lines()
        .filter_map(|line| line.strip_prefix("- `"))
        .filter_map(|line| line.split_once('`').map(|(namespace, _)| namespace.to_string()))
        .collect()
}

fn assert_local_verification_commands(commands: &str) {
    for command in [
        "CARGO_TARGET_DIR=artifacts/cargo-target cargo check -p bijux-dna-api --no-default-features",
        "CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-api --test boundaries --no-default-features",
        "CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-api --test schemas --no-default-features",
        "CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-api --test contracts --no-default-features",
    ] {
        assert!(
            commands.contains(command),
            "COMMANDS.md must list focused local verification command `{command}`"
        );
    }
}
