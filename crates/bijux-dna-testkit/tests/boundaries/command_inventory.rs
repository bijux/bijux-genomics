#[test]
fn command_inventory_lists_all_testkit_operations() {
    let commands = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/COMMANDS.md"),
    )
    .unwrap_or_else(|err| panic!("read docs/COMMANDS.md: {err}"));

    assert_eq!(
        command_names(&commands),
        expected_command_names(),
        "docs/COMMANDS.md must be the exact SSOT for testkit operations"
    );

    assert_eq!(
        verification_commands(&commands),
        expected_verification_commands(),
        "docs/COMMANDS.md must list the exact local verification commands for this crate"
    );
}

fn command_names(commands: &str) -> std::collections::BTreeSet<String> {
    commands
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            let first_cell = line.strip_prefix("| `")?.split_once('`')?.0;
            Some(first_cell.to_string())
        })
        .collect()
}

fn expected_command_names() -> std::collections::BTreeSet<String> {
    [
        "assert-json-schema-like",
        "assert-json-stable",
        "assert-stable-ordering",
        "build-snapshot-name",
        "create-fixed-clock",
        "create-fixed-rng",
        "create-test-paths",
        "create-test-temp-path",
        "create-test-tempdir",
        "derive-test-path-child",
        "install-snapshot-env",
        "list-directory-sorted",
        "load-fixture-json",
        "load-fixture-text",
        "read-fixed-clock",
        "read-policy-text",
        "read-test-path-root",
        "resolve-test-path-under-root",
        "resolve-workspace-root-from-manifest",
        "sanitize-snapshot-json",
        "sanitize-snapshot-text",
        "snapshot-normalize-json",
        "snapshot-normalize-text",
        "stable-json",
        "strip-json-timestamp-fields",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

fn verification_commands(commands: &str) -> std::collections::BTreeSet<String> {
    commands
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with("CARGO_TARGET_DIR=artifacts/cargo-target cargo test"))
        .map(str::to_string)
        .collect()
}

fn expected_verification_commands() -> std::collections::BTreeSet<String> {
    [
        "CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-testkit --test boundaries --no-default-features",
        "CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-testkit --test contracts --no-default-features",
        "CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-testkit --test determinism --no-default-features",
        "CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-testkit --test schemas --no-default-features",
        "CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-testkit --no-default-features",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}
