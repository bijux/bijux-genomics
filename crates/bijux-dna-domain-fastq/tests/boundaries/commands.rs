use std::path::Path;

#[test]
fn command_inventory_declares_library_only_surface() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let commands_doc = std::fs::read_to_string(root.join("docs/COMMANDS.md"))
        .unwrap_or_else(|err| panic!("read docs/COMMANDS.md: {err}"));
    let manifest = std::fs::read_to_string(root.join("Cargo.toml"))
        .unwrap_or_else(|err| panic!("read Cargo.toml: {err}"));

    assert!(
        commands_doc.contains("pure library crate"),
        "COMMANDS.md must explicitly state that this crate is library-only"
    );
    assert!(
        commands_doc.contains("There are no crate-managed CLI commands, Cargo binary targets"),
        "COMMANDS.md must list an empty managed command inventory"
    );
    for forbidden in [
        "No Cargo binary targets or `src/bin` command modules.",
        "No `src/main.rs`.",
        "No CLI parser ownership.",
        "No process spawning or runtime command execution.",
    ] {
        assert!(commands_doc.contains(forbidden), "COMMANDS.md must document `{forbidden}`");
    }
    assert!(
        !root.join("src/bin").exists(),
        "domain-fastq must not grow binary entrypoints without updating docs/COMMANDS.md"
    );
    assert!(!root.join("src/main.rs").exists(), "domain-fastq must not grow a main binary");
    assert!(!manifest.contains("[[bin]]"), "domain-fastq must not declare Cargo binaries");
}
