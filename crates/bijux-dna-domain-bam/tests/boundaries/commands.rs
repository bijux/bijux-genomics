use std::path::Path;

#[test]
fn command_inventory_declares_pure_library_surface() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let commands_doc = std::fs::read_to_string(root.join("docs/COMMANDS.md"))
        .unwrap_or_else(|err| panic!("read docs/COMMANDS.md: {err}"));
    let cargo_toml = std::fs::read_to_string(root.join("Cargo.toml"))
        .unwrap_or_else(|err| panic!("read Cargo.toml: {err}"));

    assert!(
        commands_doc.contains("owns no executable command surface"),
        "COMMANDS.md must explicitly state that this crate owns no commands"
    );
    assert!(
        commands_doc.contains("There are no crate-managed CLI commands"),
        "COMMANDS.md must list an empty command inventory"
    );
    assert!(!root.join("src/main.rs").exists(), "domain crate must not grow a binary entrypoint");
    assert!(!cargo_toml.contains("[[bin]]"), "domain crate must not declare binaries");
}
