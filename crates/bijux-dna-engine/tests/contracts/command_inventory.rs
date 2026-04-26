use std::path::Path;

#[test]
fn command_inventory_lists_all_engine_operations() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let commands_doc =
        std::fs::read_to_string(root.join("docs/COMMANDS.md")).expect("read docs/COMMANDS.md");
    let readme = std::fs::read_to_string(root.join("README.md")).expect("read README.md");

    let operations = command_operations(&commands_doc);
    let expected = [
        "create-engine",
        "execute-graph",
        "validate-engine-config",
        "cancel-execution",
        "check-cancellation",
        "observe-engine-event",
        "prepare-execution-graph",
        "execute-ordered-steps",
        "record-execution",
        "enforce-output-contract",
        "enforce-run-artifacts",
        "enforce-metrics-envelope",
    ];

    assert_eq!(
        operations, expected,
        "docs/COMMANDS.md must remain the complete engine operation inventory"
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
