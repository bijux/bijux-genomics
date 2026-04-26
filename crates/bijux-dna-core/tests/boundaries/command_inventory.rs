use std::collections::BTreeSet;
use std::path::Path;

const CORE_OPERATIONS: &[&str] = &[
    "canonical-json-bytes",
    "canonicalize-json",
    "canonicalize-parameters-json",
    "canonicalize-truth-json",
    "params-hash",
    "parameters-fingerprint",
    "input-fingerprint",
    "run-id-from-hashes",
    "parse-pipeline-id",
    "validate-pipeline-id",
    "parse-stage-id",
    "validate-stage-id",
    "parse-tool-id",
    "validate-tool-id",
    "validate-artifact-id",
    "validate-profile-id",
    "discover-fastq-files",
    "detect-fastq-path",
    "detect-gzip-path",
    "assess-input-dir",
    "write-input-assessment",
    "validate-execution-graph",
    "hash-execution-graph",
    "normalize-execution-graph",
    "topological-step-ids",
    "validate-execution-outputs",
    "query-run-index",
    "build-run-dir",
    "select-stage",
    "objective-spec",
    "parse-metric-id",
    "parse-derived-metric-id",
    "validate-metric-id",
    "validate-derived-metric-id",
    "metrics-schema-for-stage",
];

#[test]
fn command_inventory_lists_all_core_operations() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let commands = std::fs::read_to_string(root.join("docs/COMMANDS.md"))
        .unwrap_or_else(|err| panic!("read docs/COMMANDS.md: {err}"));

    assert!(
        commands
            .contains("This file is the SSOT for callable operations owned by `bijux-dna-core`."),
        "COMMANDS.md must identify itself as the core operation SSOT"
    );
    assert!(
        commands.contains("## Managed Core Operations"),
        "COMMANDS.md must expose the managed core operation inventory"
    );
    assert_eq!(
        command_operations(&commands),
        CORE_OPERATIONS.iter().map(|operation| (*operation).to_string()).collect(),
        "docs/COMMANDS.md must remain the complete core operation inventory"
    );
    assert!(
        !root.join("src/bin").exists(),
        "bijux-dna-core must not define binary command entrypoints"
    );
    assert_local_verification_commands(&commands);
}

fn command_operations(commands: &str) -> BTreeSet<String> {
    commands
        .lines()
        .filter_map(|line| line.strip_prefix("| `"))
        .filter_map(|line| line.split_once('`').map(|(operation, _)| operation.to_string()))
        .collect()
}

fn assert_local_verification_commands(commands: &str) {
    for command in [
        "CARGO_TARGET_DIR=artifacts/cargo-target cargo check -p bijux-dna-core --no-default-features",
        "CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-core --test boundaries --no-default-features",
        "CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-core --test contracts --no-default-features",
        "CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-core --test schemas --no-default-features",
        "CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-core --test semantics --no-default-features",
    ] {
        assert!(
            commands.contains(command),
            "COMMANDS.md must list focused local verification command `{command}`"
        );
    }
}
