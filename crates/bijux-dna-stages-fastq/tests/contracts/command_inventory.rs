#![allow(clippy::expect_used)]

use std::path::Path;

#[test]
fn command_inventory_lists_all_stages_fastq_operations() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let commands_doc =
        std::fs::read_to_string(root.join("docs/COMMANDS.md")).expect("read docs/COMMANDS.md");
    let manifest = std::fs::read_to_string(root.join("Cargo.toml")).expect("read Cargo.toml");
    let readme = std::fs::read_to_string(root.join("README.md")).expect("read README.md");

    let operations = command_operations(&commands_doc);
    let expected = [
        "list-fastq-contract-stages",
        "list-fastq-implemented-stages",
        "list-fastq-observer-stages",
        "classify-fastq-runtime-interpretation",
        "check-fastq-stage-support",
        "materialize-fastq-stage",
        "parse-fastq-stage-outputs",
        "build-fastq-metrics-envelope",
        "parse-fastq-observer-output",
        "write-fastq-observer-artifact",
    ];

    assert_eq!(
        operations, expected,
        "docs/COMMANDS.md must remain the complete stages-fastq operation inventory"
    );

    for operation in expected {
        assert!(
            readme.contains(&format!("`{operation}`")),
            "README.md must point to command operation `{operation}`"
        );
    }

    for forbidden in [
        "No Cargo binary targets or `src/bin` command modules.",
        "No CLI parser ownership.",
        "No process spawning or runtime command execution.",
        "No tool selection or pipeline composition commands.",
    ] {
        assert!(commands_doc.contains(forbidden), "COMMANDS.md must document `{forbidden}`");
    }
    assert!(!root.join("src/bin").exists(), "stages-fastq must not define command binaries");
    assert!(!root.join("src/main.rs").exists(), "stages-fastq must not define a main binary");
    assert!(!manifest.contains("[[bin]]"), "stages-fastq must not declare Cargo binaries");
}

fn command_operations(commands_doc: &str) -> Vec<String> {
    commands_doc
        .lines()
        .filter_map(|line| line.strip_prefix("| `"))
        .filter_map(|line| line.split_once('`').map(|(operation, _)| operation.to_string()))
        .collect()
}
