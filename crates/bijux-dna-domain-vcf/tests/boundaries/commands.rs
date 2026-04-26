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
        commands_doc.contains("There are no crate-managed CLI commands"),
        "COMMANDS.md must list an empty managed command inventory"
    );
    assert!(
        !root.join("src/bin").exists(),
        "domain-vcf must not grow binary entrypoints without updating docs/COMMANDS.md"
    );
    assert!(!root.join("src/main.rs").exists(), "domain-vcf must not grow a main binary");
    assert!(!manifest.contains("[[bin]]"), "domain-vcf must not declare Cargo binaries");
}
