use std::path::Path;

#[test]
fn command_inventory_lists_all_stages_fastq_operations() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let commands_doc =
        std::fs::read_to_string(root.join("docs/COMMANDS.md")).expect("read docs/COMMANDS.md");
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
}

fn command_operations(commands_doc: &str) -> Vec<String> {
    commands_doc
        .lines()
        .filter_map(|line| line.strip_prefix("| `"))
        .filter_map(|line| line.split_once('`').map(|(operation, _)| operation.to_string()))
        .collect()
}
